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

use std::fs::File;

use cairo::ImageSurface;
use gio::prelude::FileExt;
use glib::{
    clone,
    subclass::types::{ObjectSubclassExt, ObjectSubclassIsExt},
};
use gtk4::{FileDialog, FileFilter};

use crate::util::error_dialog;

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn open_file_dialog(&self) {
        let dialog = FileDialog::builder()
            .title("Choose a file")
            .accept_label("Open")
            .modal(true)
            .build();

        let all_files = FileFilter::new();
        all_files.set_name(Some("All Files"));
        all_files.add_pattern("*");

        let text_files = FileFilter::new();
        text_files.set_name(Some("Supported Files"));
        text_files.add_pattern("*.jpg");
        text_files.add_pattern("*.jpeg");
        text_files.add_pattern("*.jfif");
        text_files.add_pattern("*.gif");
        text_files.add_pattern("*.png");
        text_files.add_pattern("*.svg");
        text_files.add_pattern("*.svgz");
        text_files.add_pattern("*.webp");
        text_files.add_pattern("*.avif");
        text_files.add_pattern("*.heic");
        text_files.add_pattern("*.pcx");
        text_files.add_pattern("*.zip");
        text_files.add_pattern("*.mar");
        text_files.add_pattern("*.rar");
        text_files.add_pattern("*.pdf");
        text_files.add_pattern("*.epub");
        text_files.add_pattern("*.xps");

        let filters = gio::ListStore::new::<FileFilter>();
        filters.append(&text_files);
        filters.append(&all_files);
        dialog.set_filters(Some(&filters));
        dialog.set_default_filter(Some(&text_files));

        dialog.open(
            Some(&self.obj().clone()),
            None::<&gio::Cancellable>,
            clone!(
                #[weak(rename_to = this)]
                self,
                move |result| {
                    if let Ok(file) = result {
                        let path = file.path().unwrap_or_default();
                        this.open_file(&path);
                    }
                }
            ),
        );
    }

    pub fn save_image_dialog(&self, surface: ImageSurface) {
        // Create the modern GTK4 FileDialog builder
        let dialog = gtk4::FileDialog::new();
        dialog.set_title("Save Image As");
        dialog.set_initial_name(Some("untitled.png"));

        // Set up a file extension filter specifically for PNG files
        let filter = gtk4::FileFilter::new();
        filter.set_name(Some("PNG Image"));
        filter.add_suffix("png");

        let filters = gio::ListStore::new::<gtk4::FileFilter>();
        filters.append(&filter);
        dialog.set_filters(Some(&filters));

        dialog.save(
            Some(&self.obj().clone()),
            None::<&gio::Cancellable>,
            clone!(
                #[weak(rename_to = this)]
                self,
                move |result| {
                    match result {
                        Ok(gio_file) => {
                            // Extract the native system path from the GIO File wrapper
                            if let Some(path) = gio_file.path() {
                                surface.flush();
                                let mut file = match File::create(&path) {
                                    Ok(f) => f,
                                    Err(e) => {
                                        error_dialog(
                                            &*this.obj(),
                                            "Failed to create file",
                                            &format!("{e}"),
                                        );
                                        return;
                                    }
                                };
                                if let Err(e) = surface.write_to_png(&mut file) {
                                    error_dialog(
                                        &*this.obj(),
                                        "Failed to save image",
                                        &format!("{e}"),
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            // Triggered if the user hits "Cancel" or closes the window
                            println!("Save dialog dismissed or failed: {}", err);
                        }
                    }
                }
            ),
        );
    }

    pub fn save_raster_data_to_file(&self) {
        let w = self.widgets();
        match w.image_view.get_surface() {
            Some(surface) => {
                self.save_image_dialog(surface);
            }
            None => {
                error_dialog(
                    &*self.obj(),
                    "No texture available to save",
                    "Use Ctrl+Shift+S to save the onscreen presentation instead.",
                );
            }
        }
    }

    pub fn save_visible_area_to_file(&self) {
        let w = self.widgets();
        match w.image_view.imp().visible_to_surface() {
            Ok(surface) => {
                self.save_image_dialog(surface);
            }
            Err(e) => {
                eprintln!("Failed to get visible surface: {}", e);
            }
        };
    }
}
