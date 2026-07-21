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

use std::{
    collections::HashMap,
    fs::{create_dir_all, File},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
};

use super::MViewWindowImp;

use crate::{
    backends::{Backend, ImageParams, NoneBackend},
    classification::FileClassification,
    content::Content,
    file_view::{Direction, Filter, Target},
    util::{path_to_filename, texture_to_surface},
    window::TargetTime,
};
use glib::{clone, subclass::types::ObjectSubclassExt};
use gtk4::{gdk::Clipboard, prelude::WidgetExt};

impl MViewWindowImp {
    pub(super) fn on_selection_changed(&self) {
        let w = self.widgets();
        if !self.skip_loading.get() {
            if let Some((file_row, _)) = w.file_view.selected() {
                if let Some(current_selection) = self.current_selection.borrow().as_ref() {
                    if current_selection == &file_row {
                        // same as current selection, skipping
                        return;
                    }
                }

                let params = ImageParams {
                    tn_sender: Some(&w.tn_sender),
                    page_mode: &self.page_mode.get(),
                    allocation_height: self.obj().height(),
                };
                let backend = self.backend.borrow();
                self.target_store.borrow_mut().insert(
                    backend.normalized_path(),
                    TargetTime::new(&backend.reference(&file_row).into()),
                );

                let reference = backend.reference(&file_row);

                let mut content = backend.content(&reference.item, &params);
                content.sort(&self.current_sort.get().str_repr());

                let can_enter = content.can_enter();
                w.forward_button_top.set_visible(can_enter);
                w.panel.enable_enter(can_enter);

                // if reference.supports_bot() {
                //     let command = RenderBotCommand {
                //         id: 0,
                //         cmd: Commands::Image((
                //             reference,
                //             *params.page_mode,
                //             params.allocation_height,
                //         )),
                //     };
                //     w.rb_send(command);
                // }
                w.info_view.update(&content);
                if backend.is_thumbnail() {
                    w.image_view.set_content_pre(content);
                } else {
                    w.image_view.set_content(content);
                }
                w.info_view
                    .update_transform(w.image_view.rotation(), w.image_view.is_mirrored());

                *self.current_selection.borrow_mut() = Some(file_row);
            }
        }
    }

    pub(super) fn paste_image_from_clipboard(&self, clipboard: &Clipboard) {
        clipboard.read_texture_async(
            None::<&gio::Cancellable>,
            clone!(
                #[weak(rename_to = this)]
                self,
                move |result| {
                    match result {
                        Ok(Some(texture)) => {
                            println!("Successfully retrieved texture from clipboard!");
                            let image_surface = match texture_to_surface(&texture) {
                                Ok(surface) => surface,
                                Err(err) => {
                                    eprintln!("Error converting texture to Cairo surface: {}", err);
                                    return;
                                }
                            };
                            let w = this.widgets();
                            let new_backend = Box::new(NoneBackend::new());
                            let backend = this.backend.replace(<dyn Backend>::none());
                            let target = if let Some((file_row, _)) = w.file_view.selected() {
                                backend.reference(&file_row).into()
                            } else {
                                Target::First
                            };
                            new_backend.set_parent(backend, target);
                            new_backend.set_path(PathBuf::from("clipboard image"));
                            this.set_backend(new_backend, &Target::First, true);
                            let content = Content::new_surface(image_surface, None);
                            w.forward_button_top.set_visible(false);
                            w.panel.enable_enter(false);
                            w.info_view.update(&content);
                            w.image_view.set_content(content);
                            w.info_view.update_transform(
                                w.image_view.rotation(),
                                w.image_view.is_mirrored(),
                            );
                            *this.current_selection.borrow_mut() = None;
                        }
                        Ok(None) => {
                            eprintln!("Clipboard does not contain any image/texture format.");
                        }
                        Err(err) => {
                            eprintln!("Error reading texture from clipboard: {}", err);
                        }
                    }
                }
            ),
        );
    }

    pub(super) fn on_row_activated(&self) {
        // println!("on_row_activated");
        self.dir_enter();
    }

    pub fn dir_enter(&self) {
        let w = self.widgets();
        if let Some((file_row, _)) = w.file_view.selected() {
            let backend = self.backend.borrow();
            let new_backend = backend.enter(&file_row);
            drop(backend);
            if let Some(new_backend) = new_backend {
                let target_store = self.target_store.borrow();
                let target = target_store
                    .get(&new_backend.normalized_path())
                    .map(|tt| &tt.target)
                    .unwrap_or(&Target::First);
                self.set_backend(new_backend, target, false);
            }
        }
    }

    pub fn dir_leave(&self) {
        let backend = self.backend.borrow();
        if let Some((new_backend, target)) = backend.leave() {
            drop(backend);
            self.set_backend(new_backend, &target, true);
        }
    }

    pub fn open_file(&self, path: &Path) {
        // println!("navigate_to {}", path.display());
        let filename = path_to_filename(path);
        let directory = path.parent().unwrap_or_else(|| Path::new(""));
        let category = FileClassification::determine(path, path.is_dir());
        // dbg!(&filename, directory, category);
        match <dyn Backend>::new_from_path(directory) {
            Ok(backend) => {
                self.open_container.set(category.is_container());
                self.set_backend(backend, &Target::Name(filename), true);
            }
            Err(e) => {
                eprintln!("Failed to navigate to {}: {e}", path.display());
            }
        }
    }

    pub fn hop(&self, direction: Direction) {
        let w = self.widgets();

        // goto and navigate in parent
        self.skip_loading.set(true);
        self.dir_leave();
        w.file_view.navigate_item(direction, &Filter::Container, 1);

        // enter dir
        self.skip_loading.set(false);
        self.dir_enter();
    }

    fn navigation_cache_file(create_dir: bool) -> io::Result<PathBuf> {
        let mut path = dirs::config_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "config directory unavailable")
        })?;
        path.push("mview6");
        if create_dir {
            create_dir_all(&path)?;
        }
        path.push("navigation.json");
        Ok(path)
    }

    pub fn save_navigation(&self) {
        let target_store = self.target_store.borrow();

        // Get all entries and sort by timestamp (most recent first)
        let mut entries: Vec<_> = target_store.iter().collect();
        entries.sort_by_key(|b| std::cmp::Reverse(b.1.timestamp));

        // Take only the N most recent entries
        let recent_entries: HashMap<PathBuf, TargetTime> = entries
            .into_iter()
            .take(200)
            .map(|(k, v)| {
                (
                    k.clone(),
                    TargetTime {
                        target: v.target.clone(),
                        timestamp: v.timestamp,
                    },
                )
            })
            .collect();

        // Serialise and write on a background thread to avoid blocking the UI.
        std::thread::spawn(move || {
            let write = || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let path = Self::navigation_cache_file(true)?;
                let file = File::create(path)?;
                let writer = BufWriter::new(file);
                serde_json::to_writer_pretty(writer, &recent_entries)?;
                Ok(())
            };
            if let Err(e) = write() {
                eprintln!("Failed to save navigation cache: {e}");
            }
        });
    }

    /// Load entries from a JSON file
    pub fn load_navigation(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::open(Self::navigation_cache_file(false)?)?;
        let reader = BufReader::new(file);
        let loaded_data: HashMap<PathBuf, TargetTime> = serde_json::from_reader(reader)?;

        // Replace the current target_store with loaded data
        *self.target_store.borrow_mut() = loaded_data;

        Ok(())
    }
}
