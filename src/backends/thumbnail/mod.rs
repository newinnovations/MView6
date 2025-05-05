// MView6 -- Opiniated image browser written in Rust and GTK4
//
// Copyright (c) 2024 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU General Public License as published by the Free Software Foundation, either version 3
// of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

mod model;
pub mod processing;

use std::cell::{Cell, RefCell};

use super::{Backend, Image, ImageParams, Selection};
use crate::{
    category::Category,
    file_view::{Columns, Cursor, Sort},
    image::draw::thumbnail_sheet,
};
use gtk4::{prelude::TreeModelExt, Allocation, ListStore};
pub use model::{Message, TCommand, TEntry, TMessage, TReference, TResult, TResultOption, TTask};

const FOOTER: i32 = 50;
const MARGIN: i32 = 15;
const MIN_SEPARATOR: i32 = 5;

#[derive(Debug)]
pub struct Thumbnail {
    size: i32,
    width: i32,
    height: i32,
    // calculated
    separator_x: i32,
    separator_y: i32,
    capacity_x: i32,
    capacity_y: i32,
    offset_x: i32,
    offset_y: i32,
    // references
    parent: RefCell<Box<dyn Backend>>,
    parent_pos: i32,
    sort: Cell<Sort>,
}

impl Thumbnail {
    pub fn new(sheet_size: Allocation, position: i32, size: i32) -> Option<Self> {
        let width = sheet_size.width();
        let height = sheet_size.height();

        let usable_width = (width - 2 * MARGIN).clamp(0, i32::MAX);
        let usable_height = (height - MARGIN - FOOTER).clamp(0, i32::MAX);

        let capacity_x = (usable_width + MIN_SEPARATOR) / (size + MIN_SEPARATOR);
        let capacity_y = (usable_height + MIN_SEPARATOR) / (size + MIN_SEPARATOR);

        if capacity_x == 0 || capacity_y == 0 {
            return None;
        }

        let separator_x = (usable_width - capacity_x * size) / capacity_x;
        let separator_y = (usable_height - capacity_y * size) / capacity_y;

        let offset_x =
            MARGIN + (usable_width - capacity_x * (size + separator_x) + separator_x) / 2;
        let offset_y =
            MARGIN + (usable_height - capacity_y * (size + separator_y) + separator_y) / 2;

        Some(Thumbnail {
            size,
            width,
            height,
            separator_x,
            separator_y,
            capacity_x,
            capacity_y,
            offset_x,
            offset_y,
            parent: RefCell::new(<dyn Backend>::none()),
            parent_pos: position,
            sort: Default::default(),
        })
    }

    pub fn capacity(&self) -> i32 {
        self.capacity_x * self.capacity_y
    }

    pub fn startpage(&self) -> Selection {
        Selection::Index(self.parent_pos as u32 / self.capacity() as u32)
    }

    pub fn sheet(&self, page: i32) -> Vec<TTask> {
        let backend = self.parent.borrow();
        let store = backend.store();

        let mut res = Vec::<TTask>::new();

        let mut done = false;
        let mut tid = 0;
        let start = page * self.capacity();
        if let Some(iter) = store.iter_nth_child(None, start) {
            let cursor = Cursor::new(store, iter, start);
            for row in 0..self.capacity_y {
                if done {
                    break;
                }
                for col in 0..self.capacity_x {
                    let source = backend.entry(&cursor);
                    if !matches!(source.reference, TReference::None) {
                        let task = TTask::new(
                            tid,
                            self.size as u32,
                            self.offset_x + col * (self.size + self.separator_x),
                            self.offset_y + row * (self.size + self.separator_y),
                            source,
                        );
                        res.push(task);
                        tid += 1;
                    }
                    if !cursor.next() {
                        done = true;
                        break;
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

    fn path(&self) -> &str {
        "/thumbnail"
    }

    fn store(&self) -> ListStore {
        let parent_store = self.parent.borrow().store();
        let num_items = parent_store.iter_n_children(None);
        let pages = 1 + ((num_items - 1) / self.capacity()) as u32;
        let store = Columns::store();
        let cat = Category::Image;

        for page in 0..pages {
            let name = format!("Thumbnail page {:7}", page + 1);
            store.insert_with_values(
                None,
                &[
                    (Columns::Cat as u32, &cat.id()),
                    (Columns::Icon as u32, &cat.icon()),
                    (Columns::Name as u32, &name),
                    (Columns::Index as u32, &page),
                ],
            );
        }
        store
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        (self.parent.replace(<dyn Backend>::none()), Selection::None)
    }

    fn image(&self, cursor: &Cursor, params: &ImageParams) -> Image {
        let page = cursor.index();
        let caption = format!("{} of {}", page + 1, cursor.store_size());
        let image = match thumbnail_sheet(self.width, self.height, MARGIN, &caption) {
            Ok(image) => image,
            Err(_) => {
                println!("Failed to create thumbnail_sheet: should not happen");
                Default::default()
            }
        };

        let command = TCommand::new(image.id(), self.sheet(page as i32));
        let _ = params.sender.send_blocking(Message::Command(command));

        image
    }

    fn set_parent(&self, parent: Box<dyn Backend>) {
        if self.parent.borrow().is_none() {
            self.parent.replace(parent);
        }
    }

    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Selection)> {
        let x = (x as i32 - self.offset_x) / (self.size + self.separator_x);
        let y = (y as i32 - self.offset_y) / (self.size + self.separator_y);

        if x < 0 || y < 0 || x >= self.capacity_x || y >= self.capacity_y {
            return None;
        }

        let page = current.index() as i32;
        let pos = page * self.capacity() + y * self.capacity_x + x;

        let backend = self.parent.borrow();
        let store = backend.store();
        if let Some(iter) = store.iter_nth_child(None, pos) {
            let cursor = Cursor::new(store, iter, pos);
            let source = backend.entry(&cursor).reference;
            drop(backend);
            match source {
                TReference::FileReference(src) => Some((
                    self.parent.replace(<dyn Backend>::none()),
                    Selection::Name(src.filename()),
                )),
                TReference::ZipReference(src) => Some((
                    self.parent.replace(<dyn Backend>::none()),
                    Selection::Index(src.index()),
                )),
                TReference::RarReference(src) => Some((
                    self.parent.replace(<dyn Backend>::none()),
                    Selection::Name(src.selection()),
                )),
                TReference::DocReference(src) => Some((
                    self.parent.replace(<dyn Backend>::none()),
                    Selection::Index(src.index()),
                )),
                TReference::None => None,
            }
        } else {
            None
        }
    }

    fn set_sort(&self, sort: &Sort) {
        self.sort.set(*sort)
    }

    fn sort(&self) -> Sort {
        self.sort.get()
    }
}
