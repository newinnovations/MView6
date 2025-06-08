// MView6 -- Opiniated image and pdf browser written in Rust and GTK4
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

use super::{Backend, Image, ImageParams, Target};
use crate::{
    backends::thumbnail::model::TParent,
    category::Category,
    file_view::{Column, Cursor},
    image::draw::thumbnail_sheet,
};
use gtk4::{prelude::TreeModelExt, Allocation, ListStore};
use model::{Annotation, SheetDimensions, TRect};
pub use model::{Message, TCommand, TEntry, TMessage, TReference, TResult, TResultOption, TTask};

const FOOTER: i32 = 50;
const MARGIN: i32 = 15;
const MIN_SEPARATOR: i32 = 5;

#[derive(Debug)]
pub struct Thumbnail {
    dim: SheetDimensions,
    parent_backend: RefCell<Box<dyn Backend>>,
    parent_target: Target,
    parent_focus_pos: Cell<i32>,
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

        Thumbnail {
            dim: SheetDimensions {
                size,
                width,
                height,
                separator_x,
                separator_y,
                capacity_x,
                capacity_y,
                offset_x,
                offset_y,
            },
            parent_backend: RefCell::new(parent.backend), // <dyn Backend>::none()
            parent_target: parent.target,
            parent_focus_pos: parent.focus_pos.into(),
        }
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
        let store = backend.store();

        let mut res = Vec::<TTask>::new();

        let start = page * self.capacity();
        if let Some(iter) = store.iter_nth_child(None, start) {
            let cursor = Cursor::new(store, iter, start);
            for row in 0..self.dim.capacity_y {
                for col in 0..self.dim.capacity_x {
                    let source = backend.entry(&cursor);
                    if !matches!(source.reference, TReference::None) {
                        let x = self.dim.offset_x + col * (self.dim.size + self.dim.separator_x);
                        let y = self.dim.offset_y + row * (self.dim.size + self.dim.separator_y);
                        let id = row * self.dim.capacity_x + col;

                        let annotation = Annotation {
                            id,
                            position: TRect::new_i32(x, y, self.dim.size, self.dim.size),
                            name: source.name.clone(),
                            category: source.category,
                            reference: source.reference.clone(),
                        };
                        let task = TTask::new(id, self.dim.size as u32, x, y, source, annotation);
                        res.push(task);
                    }
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

    fn is_thumbnail(&self) -> bool {
        true
    }

    fn path(&self) -> PathBuf {
        Path::new("thumbnail").into()
    }

    fn store(&self) -> ListStore {
        let store = Column::empty_store();
        let capacity = self.capacity();
        let pages = if capacity > 0 {
            let parent_store = self.parent_backend.borrow().store();
            let num_items = parent_store.iter_n_children(None);
            if num_items > 0 {
                1 + ((num_items - 1) / capacity) as u32
            } else {
                1
            }
        } else {
            1
        };
        let cat = Category::Image;
        for page in 0..pages {
            let name = format!("Thumbnail page {:7}", page + 1);
            store.insert_with_values(
                None,
                &[
                    (Column::Cat as u32, &cat.id()),
                    (Column::Icon as u32, &cat.icon()),
                    (Column::Name as u32, &name),
                    (Column::Index as u32, &page),
                ],
            );
        }
        store
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        Some((
            self.parent_backend.replace(<dyn Backend>::none()),
            self.parent_target.clone(),
        ))
    }

    fn image(&self, cursor: &Cursor, params: &ImageParams) -> Image {
        let page = cursor.index() as i32;
        let capacity = self.capacity();
        if capacity > 0 {
            let focus_page = self.parent_focus_pos.get() / capacity;
            if focus_page != page {
                self.parent_focus_pos.set(page * capacity);
            }
        }
        let caption = format!("{} of {}", page + 1, cursor.store_size());
        let image = match thumbnail_sheet(self.dim.width, self.dim.height, MARGIN, &caption) {
            Ok(image) => image,
            Err(_) => {
                println!("Failed to create thumbnail_sheet: should not happen");
                Default::default()
            }
        };
        let command = TCommand::new(image.id(), page, self.sheet(page), self.dim.clone());
        let _ = params
            .sender
            .send_blocking(Message::Command(command.into()));
        image
    }

    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Target)> {
        if let Some(pos) = self.dim.abs_position(current.index() as i32, x, y) {
            let backend = self.parent_backend.borrow();
            let store = backend.store();
            if let Some(iter) = store.iter_nth_child(None, pos) {
                let cursor = Cursor::new(store, iter, pos);
                let source = backend.entry(&cursor).reference;
                drop(backend);
                match source {
                    TReference::FileReference(src) => Some((
                        self.parent_backend.replace(<dyn Backend>::none()),
                        Target::Name(src.filename()),
                    )),
                    TReference::ZipReference(src) => Some((
                        self.parent_backend.replace(<dyn Backend>::none()),
                        Target::Index(src.index()),
                    )),
                    TReference::MarReference(src) => Some((
                        self.parent_backend.replace(<dyn Backend>::none()),
                        Target::Index(src.index()),
                    )),
                    TReference::RarReference(src) => Some((
                        self.parent_backend.replace(<dyn Backend>::none()),
                        Target::Name(src.selection()),
                    )),
                    TReference::DocReference(src) => Some((
                        self.parent_backend.replace(<dyn Backend>::none()),
                        Target::Index(src.index()),
                    )),
                    TReference::None => None,
                }
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
        }
    }
}
