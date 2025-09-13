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

use glib::{clone, subclass::types::ObjectSubclassExt};
use gtk4::prelude::{GtkWindowExt, TreeSortableExt, TreeSortableExtManual, TreeViewExt, WidgetExt};

use crate::{
    backends::{thumbnail::Thumbnail, Backend},
    file_view::{model::Reference, Column, Sort, Target},
    util::path_to_filename,
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: &Target) {
        let skip_loading = self.skip_loading.get();
        self.skip_loading.set(true);

        let w = self.widgets();
        self.backend.replace(new_backend);
        let new_backend = self.backend.borrow();

        let mut sorting_store = self.sorting_store.borrow_mut();
        let can_be_sorted = new_backend.can_be_sorted();

        let new_sort = if can_be_sorted {
            let path = new_backend.normalized_path();
            if let Some(sort) = sorting_store.get(&path) {
                sort
            } else {
                sorting_store.insert(path, self.current_sort.get());
                &self.current_sort.get()
            }
        } else {
            &Sort::sort_on_category()
        };

        // let new_store = new_backend.store();
        let new_store = Column::store(new_backend.list());
        match new_sort {
            Sort::Sorted((column, order)) => new_store.set_sort_column_id(*column, *order),
            Sort::Unsorted => (),
        };

        drop(sorting_store); // set_backend may call set_backend again via file_view.goto when auto opening containers

        new_store.connect_sort_column_changed(clone!(
            #[weak(rename_to = this)]
            self,
            move |model| {
                this.on_sort_column_changed(model);
            }
        ));

        // TODO: think about title management
        let filename = path_to_filename(new_backend.path());
        if new_backend.is_doc() {
            self.obj().set_title(Some(&format!(
                "{filename} ({}) - MView6",
                new_backend.class_name()
            )));
        } else {
            self.obj().set_title(Some(&format!("{filename} - MView6")));
        }

        w.set_action_bool("thumb.show", new_backend.is_thumbnail());

        drop(new_backend);

        self.update_layout();
        w.file_view.set_model(Some(&new_store));
        w.file_view.set_sortable(can_be_sorted);
        self.skip_loading.set(skip_loading);
        w.file_view.goto(goto, &self.obj());
    }

    pub fn update_thumbnail_backend(&self) {
        let w = self.widgets();
        let backend = self.backend.borrow();
        if backend.is_thumbnail() {
            let parent = backend.get_thumb_parent();
            drop(backend);
            let thumbnail =
                Thumbnail::new(parent, w.image_view.allocation(), self.thumbnail_size.get());
            let focus_page = thumbnail.focus_page();
            self.set_backend(<dyn Backend>::thumbnail(thumbnail), &focus_page);
        }
    }

    pub fn reload(&self, goto: &Target) -> bool {
        let backend = self.backend.borrow();
        if let Some(new_backend) = backend.reload() {
            drop(backend);
            self.set_backend(new_backend, goto);
            true
        } else {
            false
        }
    }

    pub fn event_navigate(&self, reference: Reference) {
        // dbg!(&reference);
        let new_backend = <dyn Backend>::new_from_ref(&reference.backend);
        let goto: Target = if reference.item.is_none() {
            self.target_store
                .borrow()
                .get(&new_backend.normalized_path())
                .map(|tt| &tt.target)
                .unwrap_or(&Target::First)
                .clone()
        } else {
            reference.into()
        };
        // dbg!(&new_backend, &goto);
        self.set_backend(new_backend, &goto);
    }
}
