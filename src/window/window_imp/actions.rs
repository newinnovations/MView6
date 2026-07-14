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

use chrono::Datelike;
use glib::{clone, subclass::types::ObjectSubclassExt};
use gtk4::{
    prelude::{GtkWindowExt, WidgetExt},
    AboutDialog, License,
};

use crate::{
    backends::{
        pdf_engine, set_pdf_engine,
        thumbnail::{model::TParent, Thumbnail},
        Backend, PdfEngine,
    },
    classification::FileFormat,
    content::{ContentLoader, Preview},
    file_view::{Direction, Target},
    image::ZoomMode,
    util::{error_dialog, PreviewProgress, ProgressDialog},
    MviewError,
};

use super::MViewWindowImp;

impl MViewWindowImp {
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
        let page_no = if self.widgets().image_view.has_tag("help1") {
            2
        } else {
            1
        };
        self.show_help_page(page_no);
    }

    pub fn show_help_page(&self, page_no: i32) {
        let image = if page_no == 2 {
            ContentLoader::content_from_svg_data(
                include_bytes!("../../../resources/mv6-help-2.svgz"),
                Some("help2".to_string()),
            )
        } else {
            ContentLoader::content_from_svg_data(
                include_bytes!("../../../resources/mv6-help-1.svgz"),
                Some("help1".to_string()),
            )
        };
        if let Some(image) = image {
            self.widgets().image_view.set_content(image);
        }
    }

    pub fn change_zoom(&self, zoom: &str) {
        let w = self.widgets();
        w.set_action_string("zoom", zoom);
        w.image_view.set_zoom_mode(zoom.into());
    }

    pub fn toggle_zoom(&self) {
        let current_zoom = self.widgets().image_view.zoom_mode();
        if self.backend.borrow().is_thumbnail() {
            let new_size = match self.thumbnail_size.get() {
                175 => 140,
                140 => 100,
                100 => 80,
                80 => 250,
                _ => 175,
            };
            self.set_thumbnail_size(new_size);
        } else if current_zoom == ZoomMode::Max {
            self.change_zoom(ZoomMode::NoZoom.into());
        } else if current_zoom == ZoomMode::Fill {
            self.change_zoom(ZoomMode::Max.into());
        } else {
            self.change_zoom(ZoomMode::Fill.into());
        }
    }

    pub fn zoom_in(&self) {
        self.widgets().image_view.zoom_in();
    }

    pub fn zoom_out(&self) {
        self.widgets().image_view.zoom_out();
    }

    pub fn change_transparency(&self, transparency: &str) {
        let w = self.widgets();
        w.set_action_string("transparency", transparency);
        w.image_view.set_transparency_mode(transparency.into());
    }

    pub fn change_page_mode(&self, page_mode: &str) {
        dbg!(page_mode);
        self.widgets().set_action_string("page", page_mode);
        self.page_mode.set(page_mode.into());
        if self.backend.borrow().is_doc() {
            self.on_selection_changed();
        }
    }

    pub fn change_pdf_provider(&self, provider: &str) {
        self.widgets().set_action_string("pdf", provider);
        set_pdf_engine(provider.into());
        let current_backend = self.backend.borrow();
        if current_backend.is_doc() {
            let path = current_backend.path();
            drop(current_backend);
            self.open_file(&path);
        }
    }

    pub fn toggle_pdf_engine(&self) {
        match pdf_engine() {
            PdfEngine::MuPdf => self.change_pdf_provider(PdfEngine::Pdfium.into()),
            PdfEngine::Pdfium => self.change_pdf_provider(PdfEngine::MuPdf.into()),
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
        let w = self.widgets();
        let backend = self.backend.borrow();
        if !backend.is_thumbnail() {
            w.image_view.rotate(angle);
        }
    }

    pub fn toggle_thumbnail_view(&self) {
        let w = self.widgets();
        let backend = self.backend.borrow();
        if backend.can_show_thumbnails() {
            if let Some(store) = w.file_view.list_snapshot() {
                let position = if let Some((file_row, pos)) = w.file_view.selected() {
                    let target: Target = backend.reference(&file_row).into();
                    (target, pos)
                } else {
                    (Target::First, 0)
                };
                drop(backend);
                let parent = TParent {
                    backend: self.backend.replace(<dyn Backend>::none()),
                    target: position.0,
                    focus_pos: position.1,
                    store,
                };
                let thumbnail = Thumbnail::new(
                    parent,
                    w.image_view.width().max(0) as u32,
                    w.image_view.height().max(0) as u32,
                    self.thumbnail_size.get().try_into().unwrap_or(0),
                );
                let focus_page = thumbnail.focus_page();
                let thumbnail = <dyn Backend>::thumbnail(thumbnail);
                self.set_backend(thumbnail, &focus_page, false);
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

    pub fn toggle_slideshow(&self) {
        self.set_slideshow_active(!self.is_slideshow_active());
    }

    pub fn navigate_item_filter(&self, direction: Direction, count: u32) {
        let w = self.widgets();
        w.file_view
            .navigate_item(direction, &self.current_filter.borrow(), count);
    }

    pub fn measure_toggle(&self) {
        let w = self.widgets();
        w.image_view.measure_enable(!w.image_view.measure_active());
    }

    pub fn measure_move_endpoints(&self) {
        let w = self.widgets();
        w.image_view.measure_toggle_tracking();
    }

    pub fn create_preview(&self) {
        let w = self.widgets();
        let Some((file_row, _)) = w.file_view.selected() else {
            return;
        };
        let backend = self.backend.borrow();
        if !backend.is_filesystem() {
            return;
        }
        let fullpath = backend.path().join(file_row.name());
        drop(backend);

        let preview = Preview::new(FileFormat::from_path(&fullpath), &fullpath);

        let progress = ProgressDialog::new(
            &*self.obj(),
            "Creating Preview",
            &format!("Creating preview for '{}'...", file_row.name()),
        );

        // Preview creation (decoding video frames / rendering PDF pages) can be
        // slow, so it is run on a background thread to keep the UI responsive.
        // Progress and the final result are sent back to the GTK main loop
        // over an async channel.
        let (sender, receiver) = async_channel::unbounded();
        std::thread::spawn(move || {
            let progress_sender = sender.clone();
            let result = preview.create(&move |current, total| {
                let _ = progress_sender.send_blocking(PreviewProgress::Step(current, total));
            });
            let _ = sender.send_blocking(PreviewProgress::Done(result));
        });

        glib::spawn_future_local(clone!(
            #[weak(rename_to = this)]
            self,
            async move {
                while let Ok(msg) = receiver.recv().await {
                    match msg {
                        PreviewProgress::Step(current, total) => {
                            progress.set_progress(current, total);
                        }
                        PreviewProgress::Done(result) => {
                            progress.close();
                            match result {
                                Ok(()) => {
                                    this.current_selection.replace(None);
                                    this.on_selection_changed();
                                }
                                Err(error) => match error {
                                    MviewError::App(e) => {
                                        error_dialog(
                                            &*this.obj(),
                                            "Could not create preview",
                                            e.message(),
                                        );
                                    }
                                    _ => {
                                        error_dialog(
                                            &*this.obj(),
                                            "Could not create preview",
                                            &format!("{error}"),
                                        );
                                    }
                                },
                            }
                            break;
                        }
                    }
                }
            }
        ));
    }
}
