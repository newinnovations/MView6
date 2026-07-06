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

use gtk4::{
    gio,
    prelude::{Cast, ListModelExt},
};

use crate::classification::{FileClassification, FileType, Preference};

use super::model::file_row::FileRow;
use super::model::{Direction, Filter};

pub struct Cursor {
    model: gio::ListModel,
    store: Option<gio::ListStore>,
    file_row: FileRow,
    position: u32,
}

impl Cursor {
    pub fn new(
        model: gio::ListModel,
        store: Option<gio::ListStore>,
        file_row: FileRow,
        position: u32,
    ) -> Self {
        Cursor {
            model,
            store,
            file_row,
            position,
        }
    }

    fn category(&self) -> FileClassification {
        let row = self.file_row.row();
        let row_ref = row.as_ref().unwrap();
        FileClassification::new(
            FileType::from(row_ref.file_type),
            Preference::from_icon(row_ref.preference_icon()),
        )
    }

    fn set_position(&mut self, position: u32) -> Option<()> {
        self.position = position;
        self.file_row = self.model.item(position)?.downcast::<FileRow>().ok()?;
        Some(())
    }

    /// Position in the list (depends on the sorting order)
    pub fn position(&self) -> i32 {
        self.position as i32
    }

    /// Value of the index field of the row
    pub fn index(&self) -> u64 {
        self.file_row.row().as_ref().unwrap().index()
    }

    /// Value of the name field of the row
    pub fn name(&self) -> String {
        self.file_row.row().as_ref().unwrap().name.clone()
    }

    /// Value of the folder field of the row
    pub fn folder(&self) -> String {
        self.file_row.row().as_ref().unwrap().folder().to_string()
    }

    /// Value of the category field of the row (as u32)
    pub fn content_id(&self) -> u32 {
        self.file_row.row().as_ref().unwrap().file_type
    }

    /// Value of the content field of the row (as ContentType)
    pub fn content(&self) -> FileType {
        FileType::from(self.content_id())
    }

    /// Value of the preference field of the row (as Preference)
    pub fn preference(&self) -> Preference {
        Preference::from_icon(self.file_row.row().as_ref().unwrap().preference_icon())
    }

    pub fn update(&self, new_preference: Preference, new_filename: &str) {
        if let Some(ref mut row) = *self.file_row.row_mut() {
            row.set_preference(new_preference);
            row.name = new_filename.to_string();
        }
        if let Some(store) = &self.store {
            if let Some(position) = store.find(&self.file_row) {
                store.items_changed(position, 1, 1);
            }
        }
    }

    pub fn set_to_trash(&self, to_trash: bool) {
        if let Some(store) = &self.store {
            if let Some(position) = store.find(&self.file_row) {
                if let Some(row) = self.file_row.row().as_ref() {
                    let mut row = row.clone();
                    row.set_to_trash(to_trash);
                    let file_row = FileRow::new(row);
                    store.splice(position, 1, &[file_row]);
                }
            }
        } else if let Some(ref mut row) = *self.file_row.row_mut() {
            row.set_to_trash(to_trash);
        }
    }

    pub fn navigate(&mut self, direction: Direction, filter: &Filter, count: u32) -> Option<u32> {
        if count == 0 {
            return None;
        }

        let mut cnt = count;
        let n_items = self.model.n_items();

        while !filter.matches(self.category()) {
            let next_pos = match direction {
                Direction::Up => self.position.checked_sub(1),
                Direction::Down => (self.position + 1 < n_items).then_some(self.position + 1),
            }?;
            self.set_position(next_pos)?;
            if filter.matches(self.category()) {
                cnt = count - 1;
                break;
            }
        }

        if cnt == 0 {
            return Some(self.position);
        }

        let mut last_matching_pos = self.position;

        loop {
            let next_pos = match direction {
                Direction::Up => self.position.checked_sub(1),
                Direction::Down => (self.position + 1 < n_items).then_some(self.position + 1),
            };
            let pos = match next_pos {
                Some(p) => p,
                None => {
                    if count != cnt {
                        break;
                    }
                    return None;
                }
            };

            self.set_position(pos)?;

            if filter.matches(self.category()) {
                last_matching_pos = self.position;
                cnt -= 1;
            } else {
                continue;
            }

            if cnt == 0 {
                break;
            }
        }
        Some(last_matching_pos)
    }

    pub fn next(&mut self) -> bool {
        if self.position + 1 < self.model.n_items()
            && self.set_position(self.position + 1).is_some()
        {
            return true;
        }
        false
    }
}
