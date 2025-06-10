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

use super::MViewWindowImp;

use crate::{
    backends::{Backend, ImageParams},
    category::Category,
    file_view::{Direction, Filter, Target},
};
use gtk4::{TreePath, TreeViewColumn};

impl MViewWindowImp {
    pub(super) fn on_cursor_changed(&self) {
        // println!("on_cursor_changed skip={}", self.skip_loading.get());
        let w = self.widgets();
        if !self.skip_loading.get() {
            if let Some(current) = w.file_view.current() {
                let params = ImageParams {
                    sender: &w.sender,
                    page_mode: &self.page_mode.get(),
                };
                let backend = self.backend.borrow();
                self.target_store
                    .borrow_mut()
                    .insert(backend.path(), backend.entry(&current).into());
                let image = backend.image(&current, &params);
                w.info_view.update(&image);
                if backend.is_thumbnail() {
                    w.image_view.set_image_pre(image);
                } else {
                    w.image_view.set_image(image);
                }
            }
        }
    }

    pub(super) fn on_row_activated(&self, _path: &TreePath, _column: Option<&TreeViewColumn>) {
        println!("on_row_activated");
        self.dir_enter();
    }

    pub fn dir_enter(&self) {
        let w = self.widgets();
        if let Some(current) = w.file_view.current() {
            let backend = self.backend.borrow();
            let new_backend = backend.enter(&current);
            drop(backend);
            if let Some(new_backend) = new_backend {
                let target_store = self.target_store.borrow();
                let target = target_store
                    .get(&new_backend.path())
                    .unwrap_or(&Target::First);
                self.set_backend(new_backend, target);
            }
        }
    }

    pub fn dir_leave(&self) {
        let backend = self.backend.borrow();
        if let Some((new_backend, target)) = backend.leave() {
            drop(backend);
            self.set_backend(new_backend, &target);
        }
    }

    pub fn navigate_to(&self, path: &Path) {
        println!("navigate_to {}", path.display());
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let directory = path.parent().unwrap_or_else(|| Path::new(""));
        let category = Category::determine(path, path.is_dir());
        dbg!(filename, directory, category);
        let new_backend = <dyn Backend>::new(directory);
        self.open_container.set(category.is_container());
        self.set_backend(new_backend, &Target::Name(filename.to_string()));
    }

    pub fn hop(&self, direction: Direction) {
        let w = self.widgets();

        // goto and navigate in parent
        self.skip_loading.set(true);
        self.dir_leave();
        w.file_view.navigate(direction, Filter::Container, 1);

        // enter dir
        self.skip_loading.set(false);
        self.dir_enter();
    }
}
