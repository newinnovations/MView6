// MView6 -- High-performance PDF and photo viewer built with Rust and GTK4
//
// Copyright (c) 2024-2025 Martin van der Werff <github (at) newinnovations.nl>
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

use super::{Backend, Content, ImageParams, Target};
use crate::{
    backends::thumbnail::model::TParent,
    classification::{FileClassification, FileType},
    file_view::{
        model::{BackendRef, Entry, ItemRef, Row},
        Cursor,
    },
    image::draw::thumbnail_sheet,
    rect::PointD,
};
use gtk4::{prelude::TreeModelExt, Allocation, ListStore};
use model::{Annotation, SheetDimensions, TRect};
pub use model::{Message, TCommand, TMessage, TResult, TResultOption, TTask};

const FOOTER: i32 = 50;
const MARGIN: i32 = 15;
const MIN_SEPARATOR: i32 = 5;

#[derive(Debug)]
pub struct Thumbnail {
    dim: SheetDimensions,
    parent_backend: RefCell<Box<dyn Backend>>,
    parent_target: Target,
    parent_focus_pos: Cell<i32>,
    parent_store: ListStore,
    store: Vec<Row>,
}

impl Thumbnail {
    pub fn new(parent: TParent, sheet_size: Allocation, size: i32) -> Self {
        let width = sheet_size.width();
        let height = sheet_size.height();

        let usable_width = (width - 2 * MARGIN).clamp(0, i32::MAX);
        let usable_height = (height - MARGIN - FOOTER).clamp(0, i32::MAX);

        let capacity_x = (usable_width + MIN_SEPARATOR) / (size + MIN_SEPARATOR);
        let capacity_y = (usable_height + MIN_SEPARATOR) / (size + MIN_SEPARATOR);

        let separator_x = if capacity_x > 0 {
            (usable_width - capacity_x * size) / capacity_x
        } else {
            0
        };
        let separator_y = if capacity_y > 0 {
            (usable_height - capacity_y * size) / capacity_y
        } else {
            0
        };

        let offset_x =
            MARGIN + (usable_width - capacity_x * (size + separator_x) + separator_x) / 2;
        let offset_y =
            MARGIN + (usable_height - capacity_y * (size + separator_y) + separator_y) / 2;

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

        let capacity = dim.capacity() as u32;
        let num_items = parent.backend.list().len() as u32;

        Thumbnail {
            dim,
            parent_backend: RefCell::new(parent.backend), // <dyn Backend>::none()
            parent_target: parent.target,
            parent_focus_pos: parent.focus_pos.into(),
            parent_store: parent.store,
            store: Self::create_store(capacity, num_items),
        }
    }

    fn create_store(capacity: u32, num_items: u32) -> Vec<Row> {
        let mut result = Vec::new();
        let pages = if capacity > 0 {
            if num_items > 0 {
                1 + ((num_items - 1) / capacity)
            } else {
                1
            }
        } else {
            1
        };
        let classification = FileType::Image.into();
        for page in 0..pages {
            let name = format!("Thumbnail page {:7}", page + 1);
            result.push(Row::new_index(classification, name, 0, 0, page as u64));
        }
        result
    }

    pub fn capacity(&self) -> i32 {
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

    pub fn sheet(&self, page: i32) -> Vec<TTask> {
        let backend = self.parent_backend.borrow();

        let mut res = Vec::<TTask>::new();

        let start = page * self.capacity();
        if let Some(iter) = self.parent_store.iter_nth_child(None, start) {
            let cursor = Cursor::new(self.parent_store.clone(), iter, start);
            for row in 0..self.dim.capacity_y {
                for col in 0..self.dim.capacity_x {
                    let source = Entry {
                        category: FileClassification::new(cursor.content(), cursor.preference()),
                        name: cursor.name(),
                        reference: backend.reference(&cursor),
                    };
                    let x = self.dim.offset_x + col * (self.dim.size + self.dim.separator_x);
                    let y = self.dim.offset_y + row * (self.dim.size + self.dim.separator_y);
                    let id = row * self.dim.capacity_x + col;
                    let annotation = Annotation {
                        id,
                        position: TRect::new_i32(x, y, self.dim.size, self.dim.size),
                        entry: source.clone(),
                    };
                    let task = TTask::new(id, self.dim.size as u32, x, y, source, annotation);
                    res.push(task);
                    if !cursor.next() {
                        return res;
                    }
                }
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

    fn list(&self) -> &Vec<Row> {
        &self.store
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        Some((
            self.parent_backend.replace(<dyn Backend>::none()),
            self.parent_target.clone(),
        ))
    }

    fn content(&self, item: &ItemRef, params: &ImageParams) -> Content {
        let page = item.idx() as i32;
        let capacity = self.capacity();
        if capacity > 0 {
            let focus_page = self.parent_focus_pos.get() / capacity;
            if focus_page != page {
                self.parent_focus_pos.set(page * capacity);
            }
        }
        let caption = format!("{} of {}", page + 1, self.store.len());
        let image = match thumbnail_sheet(self.dim.width, self.dim.height, MARGIN, &caption) {
            Ok(image) => image,
            Err(_) => {
                println!("Failed to create thumbnail_sheet: should not happen");
                Default::default()
            }
        };
        let command = TCommand::new(image.id(), page, self.sheet(page), self.dim.clone());
        let _ = params
            .tn_sender
            .unwrap()
            .send_blocking(Message::Command(command.into()));
        image
    }

    fn click(&self, item: &ItemRef, mouse_pos: PointD) -> Option<(Box<dyn Backend>, Target)> {
        if let Some(idx) = self.dim.abs_position(item.idx() as i32, mouse_pos) {
            let backend = self.parent_backend.borrow();
            if let Some(iter) = self.parent_store.iter_nth_child(None, idx) {
                let cursor = Cursor::new(self.parent_store.clone(), iter, idx);
                let source = backend.reference(&cursor);
                drop(backend);
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
    }

    fn get_thumb_parent(&self) -> TParent {
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

    fn item_ref(&self, cursor: &Cursor) -> ItemRef {
        ItemRef::Index(cursor.index())
    }
}
