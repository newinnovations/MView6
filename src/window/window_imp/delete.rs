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

use super::toast::{Toast, ToastBuilder};
use super::MViewWindowImp;

use std::path::PathBuf;

use glib::clone;
use glib::subclass::types::ObjectSubclassExt;

use gio::prelude::FileExt;

use crate::{
    file_view::{BackendRef, Direction, FileRow, ItemRef},
    util::show_error_dialog,
};

/// A single file that is queued to be moved to the trash once the toast expires.
pub(super) struct PendingTrashItem {
    file_row: FileRow,
    file_path: PathBuf,
    name: String,
}

/// Tracks the files queued for a bulk "move to trash" action together with the
/// toast that lets the user undo it. Pressing DEL again while the toast is
/// visible appends the newly selected file to `items` and restarts the toast.
pub(super) struct PendingTrash {
    items: Vec<PendingTrashItem>,
    toast: Toast,
}

impl PendingTrash {
    fn title(&self) -> String {
        if self.items.len() == 1 {
            format!("Move '{}' to trash", self.items[0].name)
        } else {
            format!("{} files will be moved to trash", self.items.len())
        }
    }
}

impl MViewWindowImp {
    /// Returns true if there is a pending (not yet committed) trash action.
    pub fn has_pending_trash(&self) -> bool {
        self.pending_trash.borrow().is_some()
    }

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
            // Already queued for trashing (selection can't normally land on it again,
            // but guard against it just in case).
            if file_row.trash() {
                return;
            }

            file_row.set_trash(true);

            // Move the selection to the next file, respecting the active filter,
            // falling back to the previous one if this was the last match.
            if !w
                .file_view
                .navigate_item_bool(Direction::Down, &self.current_filter.borrow(), 1)
            {
                w.file_view
                    .navigate_item_bool(Direction::Up, &self.current_filter.borrow(), 1);
            }

            let mut pending = self.pending_trash.borrow_mut();
            if let Some(pending) = pending.as_mut() {
                pending.items.push(PendingTrashItem {
                    file_row,
                    file_path,
                    name,
                });
                let title = pending.title();
                pending.toast.restart(&title);
            } else {
                let items = vec![PendingTrashItem {
                    file_row,
                    file_path,
                    name: name.clone(),
                }];

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

                *pending = Some(PendingTrash { items, toast });
            }
        }
    }

    /// Called when the trash toast expires (or is dismissed after expiry). Moves
    /// every queued file to the trash and removes it from the file view.
    pub fn commit_pending_trash(&self) {
        let Some(pending) = self.pending_trash.borrow_mut().take() else {
            return;
        };

        let w = self.widgets();
        for item in &pending.items {
            let file = gio::File::for_path(&item.file_path);
            match file.trash(None::<&gio::Cancellable>) {
                Ok(()) => {
                    w.file_view.remove_row(&item.file_row);
                }
                Err(e) => {
                    item.file_row.set_trash(false);
                    show_error_dialog(
                        &*self.obj(),
                        "Error Trashing File",
                        &format!("Failed to move '{}' to trash: {}", item.name, e),
                    );
                }
            }
        }
        w.file_view
            .ensure_selected_filter(&self.current_filter.borrow());
    }

    /// Cancels a pending trash action: removes the trash icons and dismisses the
    /// toast without moving any files. Selection is left where it is, since the
    /// user may want to continue queueing more files for deletion.
    pub fn undo_pending_trash(&self) {
        let Some(pending) = self.pending_trash.borrow_mut().take() else {
            return;
        };
        for item in &pending.items {
            item.file_row.set_trash(false);
        }
        // Dismissing after `take()` is safe: `commit_pending_trash` will run but
        // find nothing left to do.
        pending.toast.dismiss();
    }
}
