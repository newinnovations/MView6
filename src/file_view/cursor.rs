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

use glib::object::IsA;
use gtk4::{
    prelude::{TreeModelExt, TreeModelExtManual},
    ListStore, TreeIter, TreeModel, TreePath,
};

use crate::classification::{FileClassification, FileType, Preference};

use super::model::{Column, Direction, Filter};

pub struct Cursor {
    pub store: ListStore,
    pub iter: TreeIter,
    pub position: i32,
}

impl Cursor {
    pub fn new(store: ListStore, iter: TreeIter, position: i32) -> Self {
        Cursor {
            store,
            iter,
            position,
        }
    }

    /// Postion in the list (depends on the sorting order)
    pub fn position(&self) -> i32 {
        self.position
    }

    /// Value of the index field of the row
    pub fn index(&self) -> u64 {
        self.store.index(&self.iter)
    }

    /// Value of the name field of the row
    pub fn name(&self) -> String {
        self.store.name(&self.iter)
    }

    /// Value of the folder field of the row
    pub fn folder(&self) -> String {
        self.store.folder(&self.iter)
    }

    /// Value of the category field of the row (as u32)
    pub fn content_id(&self) -> u32 {
        self.store.content_id(&self.iter)
    }

    /// Value of the content field of the row (as ContentType)
    pub fn content(&self) -> FileType {
        self.store.content(&self.iter)
    }

    /// Value of the preference field of the row (as Preference})
    pub fn preference(&self) -> Preference {
        self.store.preference(&self.iter)
    }

    pub fn update(&self, new_preference: Preference, new_filename: &str) {
        self.store.set(
            &self.iter,
            &[
                (Column::PrefIcon as u32, &new_preference.icon()),
                (Column::ShowPrefIcon as u32, &new_preference.show_icon()),
                (Column::Name as u32, &new_filename),
            ],
        );
    }

    pub fn navigate(&self, direction: Direction, filter: &Filter, count: u32) -> Option<TreePath> {
        let mut cnt = count;
        loop {
            let last = self.iter;
            let item_available = match direction {
                Direction::Up => self.store.iter_previous(&self.iter),
                Direction::Down => self.store.iter_next(&self.iter),
            };
            if !item_available {
                if count != cnt {
                    return Some(self.store.path(&last));
                }
                return None;
            }
            if !filter.matches(self.store.category(&self.iter)) {
                continue;
            }
            cnt -= 1;
            if cnt == 0 {
                break;
            }
        }
        Some(self.store.path(&self.iter))
    }

    pub fn next(&self) -> bool {
        self.store.iter_next(&self.iter)
    }
}

pub trait TreeModelMviewExt: IsA<TreeModel> {
    fn name(&self, iter: &TreeIter) -> String;
    fn folder(&self, iter: &TreeIter) -> String;
    fn content_id(&self, iter: &TreeIter) -> u32;
    fn category(&self, iter: &TreeIter) -> FileClassification;
    fn content(&self, iter: &TreeIter) -> FileType;
    fn preference(&self, iter: &TreeIter) -> Preference;
    fn index(&self, iter: &TreeIter) -> u64;
    fn modified(&self, iter: &TreeIter) -> u64;
    fn size(&self, iter: &TreeIter) -> u64;
}

impl<O: IsA<TreeModel>> TreeModelMviewExt for O {
    fn name(&self, iter: &TreeIter) -> String {
        self.get_value(iter, Column::Name as i32)
            .get::<String>()
            .unwrap_or_default()
    }
    fn folder(&self, iter: &TreeIter) -> String {
        self.get_value(iter, Column::Folder as i32)
            .get::<String>()
            .unwrap_or_default()
    }
    fn content_id(&self, iter: &TreeIter) -> u32 {
        self.get_value(iter, Column::ContentType as i32)
            .get::<u32>()
            .unwrap_or(FileType::Unsupported.id())
    }
    fn category(&self, iter: &TreeIter) -> FileClassification {
        FileClassification::new(self.content(iter), self.preference(iter))
    }
    fn content(&self, iter: &TreeIter) -> FileType {
        match self
            .get_value(iter, Column::ContentType as i32)
            .get::<u32>()
        {
            Ok(id) => FileType::from(id),
            Err(_) => FileType::Unsupported,
        }
    }
    fn preference(&self, iter: &TreeIter) -> Preference {
        let pref_icon = self
            .get_value(iter, Column::PrefIcon as i32)
            .get::<String>()
            .unwrap_or_default();
        Preference::from_icon(&pref_icon)
    }
    fn index(&self, iter: &TreeIter) -> u64 {
        self.get_value(iter, Column::Index as i32)
            .get::<u64>()
            .unwrap_or(0)
    }
    fn modified(&self, iter: &TreeIter) -> u64 {
        self.get_value(iter, Column::Modified as i32)
            .get::<u64>()
            .unwrap_or(0)
    }
    fn size(&self, iter: &TreeIter) -> u64 {
        self.get_value(iter, Column::Size as i32)
            .get::<u64>()
            .unwrap_or(0)
    }
}
