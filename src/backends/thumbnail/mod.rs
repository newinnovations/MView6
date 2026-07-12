// MView6 -- High-performance PDF and photo viewer built with Rust and GTK4
//
// Copyright (c) 2024-2026 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU Affero General Public License as published by the Free Software Foundation, either
// version 3 of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

pub mod model;
pub mod processing;

use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
};

use super::{Backend, ImageParams};
use crate::{
    backends::thumbnail::model::TParent,
    classification::{FileClassification, FileType},
    content::Content,
    file_view::{BackendRef, Entry, FileRow, FileStore, ItemRef, Target},
    image::thumbnail_sheet,
    rect::PointD,
};
use gtk4::{
    gio,
    prelude::{Cast, ListModelExt},
};
use model::{Annotation, SheetDimensions, TRect};
pub use model::{Message, TCommand, TMessage, TResult, TResultOption, TTask};

const FOOTER: u32 = 50;
const MARGIN: u32 = 15;
const MIN_SEPARATOR: u32 = 5;

#[derive(Debug)]
pub struct Thumbnail {
    dim: SheetDimensions,
    parent_backend: RefCell<Box<dyn Backend>>,
    parent_target: Target,
    parent_focus_pos: Cell<u32>,
    parent_store: gio::ListModel,
    store: FileStore,
}

impl Thumbnail {
    pub fn new(parent: TParent, width: u32, height: u32, size: u32) -> Self {
        let usable_width = width.saturating_sub(2 * MARGIN);
        let usable_height = height.saturating_sub(MARGIN + FOOTER);

        let capacity_x = (usable_width + MIN_SEPARATOR) / (size + MIN_SEPARATOR);
        let capacity_y = (usable_height + MIN_SEPARATOR) / (size + MIN_SEPARATOR);

        let separator_x = usable_width
            .saturating_sub(capacity_x * size)
            .checked_div(capacity_x)
            .unwrap_or(0);
        let separator_y = usable_height
            .saturating_sub(capacity_y * size)
            .checked_div(capacity_y)
            .unwrap_or(0);

        let offset_x = MARGIN
            + (usable_width.saturating_sub(capacity_x * (size + separator_x)) + separator_x) / 2;
        let offset_y = MARGIN
            + (usable_height.saturating_sub(capacity_y * (size + separator_y)) + separator_y) / 2;

        let dim = SheetDimensions {
            size,
            width,
            height,
            separator_x,
            separator_y,
            capacity_x,
            capacity_y,
            offset_x,
            offset_y,
        };

        let capacity = dim.capacity();
        let num_items = parent.backend.list().n_items();

        Thumbnail {
            dim,
            parent_backend: RefCell::new(parent.backend), // <dyn Backend>::none()
            parent_target: parent.target,
            parent_focus_pos: parent.focus_pos.into(),
            parent_store: parent.store,
            store: Self::create_store(capacity, num_items),
        }
    }

    fn create_store(capacity: u32, num_items: u32) -> FileStore {
        let store = FileRow::empty_store();
        // capacity = 10  num_items =  0..10 => pages = 1
        // capacity = 10  num_items = 11..20 => pages = 2
        // capacity = 10  num_items = 21..30 => pages = 3 ...
        let pages = if let Some(pages) = num_items.saturating_sub(1).checked_div(capacity) {
            pages + 1
        } else {
            1
        };

        let classification = FileType::Image.into();
        for page in 0..pages {
            let name = format!("Thumbnail page {:7}", page + 1);
            store.append(&FileRow::new_index(classification, name, 0, 0, page as u64));
        }
        store
    }

    pub fn capacity(&self) -> u32 {
        self.dim.capacity()
    }

    pub fn focus_page(&self) -> Target {
        let capacity = self.capacity();
        if capacity > 0 {
            Target::Index(self.parent_focus_pos.get() as u64 / capacity as u64)
        } else {
            Target::First
        }
    }

    pub fn sheet(&self, page: u32) -> Vec<TTask> {
        let backend = self.parent_backend.borrow();

        let mut res = Vec::<TTask>::new();

        let mut position = page * self.capacity();
        let num_items = self.parent_store.n_items();
        for row in 0..self.dim.capacity_y {
            for col in 0..self.dim.capacity_x {
                if position > num_items {
                    return res;
                }
                if let Some(obj) = self.parent_store.item(position) {
                    if let Ok(file_row) = obj.downcast::<FileRow>() {
                        let source = Entry {
                            category: FileClassification::new(
                                file_row.file_type(),
                                file_row.preference(),
                            ),
                            name: file_row.name(),
                            reference: backend.reference(&file_row),
                        };
                        let x = self.dim.offset_x + col * (self.dim.size + self.dim.separator_x);
                        let y = self.dim.offset_y + row * (self.dim.size + self.dim.separator_y);
                        let id = row * self.dim.capacity_x + col;
                        let annotation = Annotation {
                            id,
                            position: TRect::new_u32(x, y, self.dim.size, self.dim.size),
                            entry: source.clone(),
                        };
                        let task = TTask::new(id, self.dim.size, x, y, source, annotation);
                        res.push(task);
                    }
                }
                position += 1;
            }
        }

        res
    }
}

impl Backend for Thumbnail {
    fn class_name(&self) -> &str {
        "Thumbnail"
    }

    fn path(&self) -> PathBuf {
        Path::new("thumbnail").into()
    }

    fn list(&self) -> FileStore {
        self.store.clone()
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        // Moves the parent backend out (one-shot): subsequent calls will return NoneBackend.
        Some((
            self.parent_backend.replace(<dyn Backend>::none()),
            self.parent_target.clone(),
        ))
    }

    fn content(&self, item: &ItemRef, params: &ImageParams) -> Content {
        let page = item.idx() as u32;
        let capacity = self.capacity();
        if let Some(focus_page) = self.parent_focus_pos.get().checked_div(capacity) {
            if focus_page != page {
                self.parent_focus_pos.set(page * capacity);
            }
        }
        let caption = format!("{} of {}", page + 1, self.store.n_items());
        let image = match thumbnail_sheet(self.dim.width, self.dim.height, MARGIN, &caption) {
            Ok(image) => image,
            Err(_) => {
                println!("Failed to create thumbnail_sheet: should not happen");
                Default::default()
            }
        };
        let command = TCommand::new(image.id(), page, self.sheet(page), self.dim.clone());
        if let Some(sender) = params.tn_sender {
            let _ = sender.send_blocking(Message::Command(command.into()));
        } else {
            eprintln!("Thumbnail content requested without a thumbnail sender");
        }
        image
    }

    fn click(&self, item: &ItemRef, mouse_pos: PointD) -> Option<(Box<dyn Backend>, Target)> {
        if let Some(idx) = self.dim.abs_position(item.idx() as u32, mouse_pos) {
            let backend = self.parent_backend.borrow();
            if let Some(obj) = self.parent_store.item(idx) {
                if let Ok(file_row) = obj.downcast::<FileRow>() {
                    let source = backend.reference(&file_row);
                    drop(backend);
                    // Moves the parent backend out (one-shot).
                    Some((
                        self.parent_backend.replace(<dyn Backend>::none()),
                        source.into(),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_thumb_parent(&self) -> TParent {
        // Moves the parent backend out (one-shot).
        TParent {
            backend: self.parent_backend.replace(<dyn Backend>::none()),
            target: self.parent_target.clone(),
            focus_pos: self.parent_focus_pos.get(),
            store: self.parent_store.clone(),
        }
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::Thumbnail //(self.parent_backend.borrow().reference(cursor))
    }
}
