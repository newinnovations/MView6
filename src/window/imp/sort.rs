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

use super::MViewWindowImp;

use crate::file_view::{Column, Sort};
use glib::{clone, idle_add_local, ControlFlow};
use gtk4::{
    prelude::{TreeSortableExtManual, TreeViewExt},
    ListStore, SortColumn, SortType, TreeViewColumn,
};

impl MViewWindowImp {
    pub fn change_sort(&self, sort_col: Column) {
        let backend = self.backend.borrow();
        if backend.can_be_sorted() {
            let store = backend.store();
            let new_sort_column = SortColumn::Index(sort_col as u32);
            let current_sort = store.sort_column_id();
            let new_direction = match current_sort {
                Some((current_column, current_direction)) => {
                    if current_column.eq(&new_sort_column) {
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
            store.set_sort_column_id(new_sort_column, new_direction);
        }
    }

    /// Called as a consequence of change_sort or by clicking the TreeView headers
    pub fn on_sort_column_changed(&self, model: &ListStore) {
        let previous_sort = self.current_sort.get();
        if let Some((new_column, new_order)) = model.sort_column_id() {
            self.current_sort.set(Sort::new(new_column, new_order));
            let path = self.backend.borrow().normalized_path();
            self.sorting_store
                .borrow_mut()
                .insert(path, self.current_sort.get());
            if let Sort::Sorted((previous_column, _)) = previous_sort {
                if !previous_column.eq(&new_column)
                    && new_column == SortColumn::Index(Column::Modified as u32)
                {
                    model.set_sort_column_id(
                        SortColumn::Index(Column::Modified as u32),
                        SortType::Descending,
                    )
                }
            }
            self.bring_entry_into_view();
        }
    }

    pub fn bring_entry_into_view(&self) {
        idle_add_local(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            ControlFlow::Break,
            move || {
                let w = this.widgets();
                let (tree_path, _) = w.file_view.cursor();
                if let Some(tree_path) = tree_path {
                    let old = this.skip_loading.replace(true);
                    w.file_view
                        .set_cursor(&tree_path, None::<&TreeViewColumn>, false);
                    this.skip_loading.set(old);
                }
                ControlFlow::Break
            }
        ));
    }
}
