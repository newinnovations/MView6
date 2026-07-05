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

use gio::prelude::FileExt;
use glib::clone;
use glib::subclass::types::ObjectSubclassExt;
use gtk4::prelude::{DialogExt, GtkWindowExt, WidgetExt};

use crate::{
    backends::Backend,
    file_view::{BackendRef, Direction, ItemRef, Target},
    util::show_error_dialog,
};

impl MViewWindowImp {
    pub fn delete_current_file(&self, permanent: bool) {
        let w = self.widgets();
        let current = match w.file_view.current() {
            Some(c) => c,
            None => return,
        };

        let (dir_path, old_name) = {
            let backend = self.backend.borrow();
            if !backend.is_filesystem() {
                return;
            }

            let reference = backend.reference(&current);
            match &reference.backend {
                BackendRef::FileSystem(dir) => {
                    if let ItemRef::String(name) = &reference.item {
                        (dir.clone(), name.clone())
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        };
        let old_file_path = dir_path.join(&old_name);

        if permanent {
            // Ask for confirmation
            let dialog = gtk4::MessageDialog::new(
                Some(&*self.obj()),
                gtk4::DialogFlags::MODAL,
                gtk4::MessageType::Question,
                gtk4::ButtonsType::YesNo,
                "Permanently Delete?",
            );
            dialog.set_secondary_text(Some(&format!(
                "Are you sure you want to permanently delete '{}'?",
                old_name
            )));
            dialog.connect_response(clone!(
                #[weak(rename_to = this)]
                self,
                move |dialog, response| {
                    if response == gtk4::ResponseType::Yes {
                        let w = this.widgets();
                        let next_target = if w.file_view.navigate_item_bool(
                            Direction::Down,
                            &this.current_filter.borrow(),
                            1,
                        ) {
                            w.file_view.current().map(|c| Target::Name(c.name()))
                        } else if w.file_view.navigate_item_bool(
                            Direction::Up,
                            &this.current_filter.borrow(),
                            1,
                        ) {
                            w.file_view.current().map(|c| Target::Name(c.name()))
                        } else {
                            None
                        };

                        if next_target.is_none() {
                            this.set_backend(<dyn Backend>::none(), &Target::First, true);
                        }

                        let result = if old_file_path.is_dir() {
                            std::fs::remove_dir_all(&old_file_path)
                        } else {
                            std::fs::remove_file(&old_file_path)
                        };

                        match result {
                            Ok(()) => {
                                if let Some(target) = next_target {
                                    this.reload(&target, false);
                                }
                            }
                            Err(e) => {
                                show_error_dialog(
                                    &*this.obj(),
                                    "Error Deleting File",
                                    &format!("Failed to permanently delete file: {}", e),
                                );
                                this.reload(&Target::Name(old_name.clone()), false);
                            }
                        }
                    }
                    dialog.close();
                }
            ));
            dialog.show();
        } else {
            // Move to trash without confirmation
            let next_target =
                if w.file_view
                    .navigate_item_bool(Direction::Down, &self.current_filter.borrow(), 1)
                {
                    w.file_view.current().map(|c| Target::Name(c.name()))
                } else if w.file_view.navigate_item_bool(
                    Direction::Up,
                    &self.current_filter.borrow(),
                    1,
                ) {
                    w.file_view.current().map(|c| Target::Name(c.name()))
                } else {
                    None
                };

            if next_target.is_none() {
                self.set_backend(<dyn Backend>::none(), &Target::First, true);
            }

            let file = gio::File::for_path(&old_file_path);
            match file.trash(None::<&gio::Cancellable>) {
                Ok(()) => {
                    if let Some(target) = next_target {
                        self.reload(&target, false);
                    }
                }
                Err(e) => {
                    show_error_dialog(
                        &*self.obj(),
                        "Error Trashing File",
                        &format!("Failed to move file to trash: {}", e),
                    );
                    self.reload(&Target::Name(old_name), false);
                }
            }
        }
    }
}
