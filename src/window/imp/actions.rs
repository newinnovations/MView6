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

use chrono::Datelike;
use gio::prelude::FileExt;
use glib::subclass::types::ObjectSubclassExt;
use gtk4::{
    prelude::{DialogExt, FileChooserExt, FileChooserExtManual, GtkWindowExt, WidgetExt},
    AboutDialog, FileChooserAction, FileChooserDialog, FileFilter, License, ResponseType,
};

use crate::{
    backends::{thumbnail::Thumbnail, Backend},
    file_view::{Sort, Target},
    image::provider::ImageLoader,
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn open_file(&self) {
        // Create the file open dialog
        let dialog = FileChooserDialog::new(
            Some("Choose a file"),
            Some(&self.obj().clone()),
            FileChooserAction::Open,
            &[
                ("Cancel", ResponseType::Cancel),
                ("Open", ResponseType::Accept),
            ],
        );

        // Create file filters
        let all_files = FileFilter::new();
        all_files.set_name(Some("All Files"));
        all_files.add_pattern("*");

        let text_files = FileFilter::new();
        text_files.set_name(Some("Text Files"));
        text_files.add_pattern("*.txt");
        text_files.add_pattern("*.md");

        // Add filters to the dialog
        dialog.add_filter(&text_files);
        dialog.add_filter(&all_files);

        // Set default folder (optional)
        _ = dialog.set_current_folder(Some(&gio::File::for_path("/home")));

        // Show the dialog and handle the response
        dialog.connect_response(|dialog, response| {
            if response == ResponseType::Accept {
                // Get the selected file
                if let Some(file) = dialog.file() {
                    if let Some(path) = file.path() {
                        println!("Selected file: {:?}", path);
                        // Here you would handle the file (open it, read it, etc.)
                    }
                }
            }
            dialog.destroy();
        });

        dialog.show();
    }

    pub fn show_about_dialog(&self) {
        let dialog = AboutDialog::builder()
            .transient_for(&self.obj().clone())
            .modal(true)
            .program_name("MView6")
            .version(env!("CARGO_PKG_VERSION")) // Get version from Cargo.toml
            .logo_icon_name("mview6") // This will load the icon from resources
            .authors(vec![env!("CARGO_PKG_AUTHORS")]) // Get authors from Cargo.toml
            .copyright(format!(
                "© {} {}",
                chrono::Local::now().year(),
                env!("CARGO_PKG_AUTHORS")
            ))
            .comments(env!("CARGO_PKG_DESCRIPTION"))
            .license_type(License::Agpl30)
            .website(env!("CARGO_PKG_REPOSITORY")) // Get repository URL from Cargo.toml
            .website_label("Visit source repository")
            .build();
        dialog.present();
    }

    pub fn quit(&self) {
        self.obj().close();
    }

    pub fn show_help(&self) {
        let w = self.widgets();
        let image = if w.image_view.has_tag("help1") {
            ImageLoader::image_from_svg_data(
                include_bytes!("../../../resources/mv6-help-2.svgz"),
                Some("help2".to_string()),
            )
        } else {
            ImageLoader::image_from_svg_data(
                include_bytes!("../../../resources/mv6-help-1.svgz"),
                Some("help1".to_string()),
            )
        };
        if let Some(image) = image {
            w.image_view.set_image(image);
        }
    }

    pub fn change_zoom(&self, zoom: &str) {
        let w = self.widgets();
        w.set_action_string("zoom", zoom);
        w.image_view.set_zoom_mode(zoom.into());
    }

    pub fn change_page_mode(&self, page_mode: &str) {
        dbg!(page_mode);
        self.widgets().set_action_string("page", page_mode);
        self.page_mode.set(page_mode.into());
        if self.backend.borrow().is_doc() {
            self.on_cursor_changed();
        }
    }

    pub fn toggle_fullscreen(&self) {
        let w = self.widgets();
        let is_fullscreen = if self.fullscreen.get() {
            self.obj().unfullscreen();
            false
        } else {
            self.show_files_widget(false);
            self.obj().fullscreen();
            true
        };
        self.fullscreen.set(is_fullscreen);
        w.set_action_bool("fullscreen", is_fullscreen);
    }

    pub fn toggle_pane_files(&self) {
        self.show_files_widget(!self.widgets().file_widget.is_visible());
    }

    pub fn toggle_pane_info(&self) {
        if !self.backend.borrow().is_thumbnail() {
            self.show_info_widget(!self.widgets().info_widget.is_visible());
        }
    }

    pub fn rotate_image(&self, angle: i32) {
        self.widgets().image_view.rotate(angle);
    }

    pub fn toggle_thumbnail_view(&self) {
        let w = self.widgets();
        let backend = self.backend.borrow();
        if backend.is_container() {
            let position = if let Some(cursor) = w.file_view.current() {
                let target: Target = backend.entry(&cursor).into();
                (target, cursor.position())
            } else {
                (Target::First, 0)
            };
            drop(backend);
            if let Some(thumbnail) = Thumbnail::new(
                w.image_view.allocation(),
                position,
                self.thumbnail_size.get(),
            ) {
                let startpage = thumbnail.startpage();
                let new_backend = <dyn Backend>::thumbnail(thumbnail);
                new_backend.set_sort(&Sort::sort_on_category());
                self.set_backend(new_backend, startpage, true);
                self.show_info_widget(false);
            }
        } else if backend.is_thumbnail() {
            drop(backend);
            self.dir_leave();
        }
    }

    pub fn set_thumbnail_size(&self, new_size: i32) {
        self.widgets()
            .set_action_string("thumb.size", &new_size.to_string());
        self.thumbnail_size.set(new_size);
        self.update_thumbnail_backend()
    }
}
