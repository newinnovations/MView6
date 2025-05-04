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

use glib::object::IsA;
use gtk4::{
    prelude::{TreeModelExt, TreeModelExtManual, TreeSortableExtManual},
    ListStore, SortColumn, SortType, TreeIter, TreeModel, TreePath,
};

use crate::category::Category;

use super::model::{Columns, Direction, Filter};

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
    pub fn index(&self) -> u32 {
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
    pub fn category_id(&self) -> u32 {
        self.store.category_id(&self.iter)
    }

    /// Value of the category field of the row (as Category)
    pub fn category(&self) -> Category {
        self.store.category(&self.iter)
    }

    pub fn store_size(&self) -> i32 {
        self.store.iter_n_children(None)
    }

    pub fn update(&self, new_category: Category, new_filename: &str) {
        self.store.set(
            &self.iter,
            &[
                (Columns::Cat as u32, &new_category.id()),
                (Columns::Icon as u32, &new_category.icon()),
                (Columns::Name as u32, &new_filename),
            ],
        );
    }

    pub fn set_sort_column(&self, new_column: SortColumn) {
        let current_sort = self.store.sort_column_id();
        let new_direction = match current_sort {
            Some((current_column, current_direction)) => {
                if current_column.eq(&new_column) {
                    match current_direction {
                        SortType::Ascending => SortType::Descending,
                        _ => SortType::Ascending,
                    }
                } else {
                    SortType::Ascending
                }
            }
            None => SortType::Ascending,
        };
        self.store.set_sort_column_id(new_column, new_direction);
    }

    pub fn navigate(&self, direction: Direction, filter: Filter, count: i32) -> Option<TreePath> {
        let mut cnt = count;
        loop {
            let last = self.iter;
            let result = match direction {
                Direction::Up => self.store.iter_previous(&self.iter),
                Direction::Down => self.store.iter_next(&self.iter),
            };
            if !result {
                if count != cnt {
                    return Some(self.store.path(&last));
                }
                return None;
            }

            let cat = self.store.category(&self.iter);

            let skip = match filter {
                Filter::None => false,
                Filter::Image => cat != Category::Image && cat != Category::Favorite,
                Filter::Favorite => cat != Category::Favorite,
                Filter::Container => {
                    cat != Category::Folder && cat != Category::Archive && cat != Category::Pdf
                }
            };

            if skip {
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
    fn category_id(&self, iter: &TreeIter) -> u32;
    fn category(&self, iter: &TreeIter) -> Category;
    fn index(&self, iter: &TreeIter) -> u32;
    fn modified(&self, iter: &TreeIter) -> u64;
    fn size(&self, iter: &TreeIter) -> u64;
}

impl<O: IsA<TreeModel>> TreeModelMviewExt for O {
    fn name(&self, iter: &TreeIter) -> String {
        self.get_value(iter, Columns::Name as i32)
            .get::<String>()
            .unwrap_or_default()
    }
    fn folder(&self, iter: &TreeIter) -> String {
        self.get_value(iter, Columns::Folder as i32)
            .get::<String>()
            .unwrap_or_default()
    }
    fn category_id(&self, iter: &TreeIter) -> u32 {
        self.get_value(iter, Columns::Cat as i32)
            .get::<u32>()
            .unwrap_or(Category::Unsupported.id())
    }
    fn category(&self, iter: &TreeIter) -> Category {
        match self.get_value(iter, Columns::Cat as i32).get::<u32>() {
            Ok(id) => Category::from(id),
            Err(_) => Default::default(),
        }
    }
    fn index(&self, iter: &TreeIter) -> u32 {
        self.get_value(iter, Columns::Index as i32)
            .get::<u32>()
            .unwrap_or(0)
    }
    fn modified(&self, iter: &TreeIter) -> u64 {
        self.get_value(iter, Columns::Modified as i32)
            .get::<u64>()
            .unwrap_or(0)
    }
    fn size(&self, iter: &TreeIter) -> u64 {
        self.get_value(iter, Columns::Size as i32)
            .get::<u64>()
            .unwrap_or(0)
    }
}
