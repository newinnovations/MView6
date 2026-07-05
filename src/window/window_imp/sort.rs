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

use super::MViewWindowImp;

use crate::file_view::{Column, FileView, Sort};
use glib::{clone, idle_add_local, ControlFlow};
use gtk4::{prelude::Cast, SortType};

impl MViewWindowImp {
    pub fn change_sort(&self, sort_col: Column, file_view: &FileView) {
        let backend = self.backend.borrow();
        if backend.can_be_sorted() {
            file_view.change_sort(sort_col);
        }
    }

    /// Called as a consequence of change_sort or by clicking the ColumnView headers
    pub fn on_sort_column_changed(&self) {
        let w = self.widgets();
        let previous_sort = self.current_sort.get();
        if let Some((sort_col, new_order)) = w.file_view.current_sort() {
            let new_sort = Sort::new(sort_col, new_order);
            self.current_sort.set(new_sort);
            if let Sort::Sorted((previous_column, _)) = previous_sort {
                if previous_column != sort_col
                    && sort_col == Column::Modified
                    && new_order != SortType::Descending
                {
                    w.file_view.set_sort(Column::Modified, SortType::Descending);
                    // We will get back in `on_sort_column_changed` because the order change
                    return;
                }
            }
            let path = self.backend.borrow().normalized_path();
            self.sorting_store
                .borrow_mut()
                .insert(path, self.current_sort.get());
            self.bring_entry_into_view();
            w.image_view.on_sort_changed(&new_sort.str_repr());
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
                if let Some(selection_model) = w
                    .file_view
                    .model()
                    .and_then(|m| m.downcast::<gtk4::SingleSelection>().ok())
                {
                    let selected = selection_model.selected();
                    if selected != gtk4::INVALID_LIST_POSITION {
                        let old = this.skip_loading.replace(true);
                        w.file_view.select_index(selected);
                        this.skip_loading.set(old);
                    }
                }
                ControlFlow::Break
            }
        ));
    }
}
