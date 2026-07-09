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

use glib::clone;
use glib::subclass::types::ObjectSubclassExt;

use crate::{
    file_view::{BackendRef, ItemRef},
    util::show_error_dialog,
    window::window_imp::toast::ToastBuilder,
};

impl MViewWindowImp {
    pub fn delete_current_file(&self, permanent: bool) {
        let w = self.widgets();
        let Some((file_row, _)) = w.file_view.selected() else {
            return;
        };

        let (dir_path, name) = {
            let backend = self.backend.borrow();
            if !backend.is_filesystem() {
                return;
            }

            let reference = backend.reference(&file_row);
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
        let file_path = dir_path.join(&name);

        if permanent {
            let dialog = gtk4::AlertDialog::builder()
                .modal(true)
                .message("Permanently Delete?")
                .detail(format!(
                    "Are you sure you want to permanently delete '{}'?",
                    name
                ))
                .buttons(["No", "Yes"])
                .cancel_button(0)
                .default_button(0)
                .build();
            dialog.choose(
                Some(&*self.obj()),
                None::<&gio::Cancellable>,
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    move |response| {
                        if response == Ok(1) {
                            println!(
                                "Permanently deleting file: {:?}, {:?}",
                                file_path,
                                file_row.name()
                            );

                            let result = if file_path.is_dir() {
                                std::fs::remove_dir_all(&file_path)
                            } else {
                                std::fs::remove_file(&file_path)
                            };

                            match result {
                                Ok(()) => {
                                    let w = this.widgets();
                                    w.file_view.remove_row(&file_row);
                                    w.file_view
                                        .ensure_selected_filter(&this.current_filter.borrow());
                                }
                                Err(e) => {
                                    show_error_dialog(
                                        &*this.obj(),
                                        "Error Deleting File",
                                        &format!("Failed to permanently delete file: {}", e),
                                    );
                                }
                            }
                        }
                    }
                ),
            );
        } else {
            println!("Trashing file: {:?}, {:?}", file_path, file_row.name());

            file_row.set_trash(true);

            let toast = ToastBuilder::new(&format!("Move '{}' to trash", name))
                .button_label("Undo")
                .action_name("win.trash.undo")
                .on_dismissed(clone!(
                    #[weak(rename_to = this)]
                    self,
                    move |_| {
                        this.commit_pending_trash();
                    }
                ))
                .build();

            self.widgets().toast_overlay.add_toast(&toast);

            // Move to trash without confirmation
            // let next_target =
            //     if w.file_view
            //         .navigate_item_bool(Direction::Down, &self.current_filter.borrow(), 1)
            //     {
            //         w.file_view.current().map(|c| Target::Name(c.file_row().name()))
            //     } else if w.file_view.navigate_item_bool(
            //         Direction::Up,
            //         &self.current_filter.borrow(),
            //         1,
            //     ) {
            //         w.file_view.current().map(|c| Target::Name(c.file_row().name()))
            //     } else {
            //         None
            //     };

            // if next_target.is_none() {
            //     self.set_backend(<dyn Backend>::none(), &Target::First, true);
            // }

            // let file = gio::File::for_path(&old_file_path);
            // match file.trash(None::<&gio::Cancellable>) {
            //     Ok(()) => {
            //         if let Some(target) = next_target {
            //             self.reload(&target, false);
            //         }
            //     }
            //     Err(e) => {
            //         show_error_dialog(
            //             &*self.obj(),
            //             "Error Trashing File",
            //             &format!("Failed to move file to trash: {}", e),
            //         );
            //         self.reload(&Target::Name(old_name), false);
            //     }
            // }
        }
    }

    pub fn commit_pending_trash(&self) {
        println!("Committing pending trash...");
    }

    pub fn undo_pending_trash(&self) {
        println!("Undoing pending trash...");
    }
}
