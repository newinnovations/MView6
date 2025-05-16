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

use std::path::Path;

use glib::{clone, subclass::types::ObjectSubclassExt};
use gtk4::prelude::{GtkWindowExt, TreeSortableExt, TreeSortableExtManual, TreeViewExt, WidgetExt};

use crate::{
    backends::{thumbnail::Thumbnail, Backend},
    file_view::{Sort, Target},
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn set_backend(&self, new_backend: Box<dyn Backend>, goto: Target, set_parent: bool) {
        let skip_loading = self.skip_loading.get();
        self.skip_loading.set(true);

        let w = self.widgets();
        let parent_backend = self.backend.replace(new_backend);
        let new_backend = self.backend.borrow();

        if set_parent {
            // dbg!(new_backend.class_name(), parent_backend.class_name());
            parent_backend.set_sort(&self.current_sort.get());
            if !parent_backend.is_bookmarks() {
                new_backend.set_parent(parent_backend);
            }
        }

        let new_sort = match new_backend.sort() {
            Sort::Sorted(sort) => {
                let sort = Sort::Sorted(sort);
                self.current_sort.set(sort);
                sort
            }
            Sort::Unsorted => {
                let sort = self.current_sort.get();
                new_backend.set_sort(&sort);
                sort
            }
        };

        let new_store = new_backend.store();
        match new_sort {
            Sort::Sorted((column, order)) => new_store.set_sort_column_id(column, order),
            Sort::Unsorted => (),
        };

        new_store.connect_sort_column_changed(clone!(
            #[weak(rename_to = this)]
            self,
            move |model| {
                Sort::on_sort_column_changed(model, &this.current_sort);
            }
        ));

        // TODO: think about title management
        let path = Path::new(new_backend.path());
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        self.obj().set_title(Some(&format!("{filename} - MView6")));

        w.set_action_bool("thumb.show", new_backend.is_thumbnail());

        drop(new_backend);

        self.update_layout();
        w.file_view.set_model(Some(&new_store));
        self.skip_loading.set(skip_loading);
        w.file_view.goto(&goto, &self.obj());
    }

    pub fn update_thumbnail_backend(&self) {
        let w = self.widgets();
        let backend = self.backend.borrow();
        if backend.is_thumbnail() {
            if let Some(thumbnail) = Thumbnail::new(
                w.image_view.allocation(),
                backend.position(),
                self.thumbnail_size.get(),
            ) {
                let (parent_backend, _) = backend.leave();
                drop(backend);
                self.backend.replace(parent_backend);
                let startpage = thumbnail.startpage();
                let new_backend = <dyn Backend>::thumbnail(thumbnail);
                self.set_backend(new_backend, startpage, true);
            }
        }
    }
}
