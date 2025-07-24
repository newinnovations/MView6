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

use std::time::Duration;

use glib::{clone, ControlFlow};
use gtk4::prelude::WidgetExt;

use crate::{
    backends::ImageParams,
    image::view::data::{QUALITY_HIGH, QUALITY_LOW},
    util::{has_changed_by_percentage, remove_source_id},
};

use super::MViewWindowImp;

const DELAY_CANVAS_RESIZED: u64 = 100;
const DELAY_HQ_REDRAW: u64 = 100;

impl MViewWindowImp {
    /// 1. Canvas size changes
    ///
    /// caused by
    /// - fullscreen transitions
    /// - panes show/hide
    /// - manual window resize
    ///
    /// actions
    /// - rerender content
    ///   - pdf
    ///   - thumbnail sheet
    /// - redraw in view (apply zoom)
    ///   - regular images
    ///
    /// notes:
    /// - use height of the window to determine rerender (eg > 10%)
    pub fn event_canvas_resized(&self, _width: i32, height: i32) {
        self.cancel_canvas_resized();
        self.schedule_canvas_resized(height);
        let image_view = &self.widgets().image_view;
        image_view.apply_zoom();
        image_view.redraw(QUALITY_LOW);
    }

    /// 2. View position in canvas changes
    ///
    /// caused by
    /// - drag
    ///
    /// actions
    /// - rerender overlay (if present)
    ///   - pdf
    /// - redraw in view
    ///   - regular images and unzoomed pdf
    ///
    /// 3. Zoom changes
    ///
    /// caused by
    /// - zoom setting change
    /// - mouse wheel
    ///
    /// actions:
    /// - (re)render overlay
    ///   - pdf
    /// - redraw in view
    ///   - regular images
    pub fn event_hq_redraw(&self, delayed: bool) {
        self.cancel_hq_redraw();
        if delayed {
            self.schedule_hq_redraw();
            self.widgets().image_view.redraw(QUALITY_LOW);
        } else {
            self.hq_redraw();
        }
    }

    fn re_render_image(&self, height: i32) {
        let w = self.widgets();
        if let Some(current) = w.file_view.current() {
            // println!("re-render");
            let params = ImageParams {
                sender: &w.sender,
                page_mode: &self.page_mode.get(),
                allocation_height: height,
            };
            let backend = self.backend.borrow();
            let image = backend.image(&current, &params);
            w.info_view.update(&image);
            w.image_view.set_image(image);
        }
    }

    fn cancel_canvas_resized(&self) {
        if let Some(id) = self.canvas_resized_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {e}");
            }
        }
    }

    fn schedule_canvas_resized(&self, height: i32) {
        self.canvas_resized_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_millis(DELAY_CANVAS_RESIZED),
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.canvas_resized_timeout_id.replace(None);
                        let w = this.widgets();
                        let backend = this.backend.borrow();
                        if backend.is_doc() {
                            if has_changed_by_percentage(
                                this.current_height.get() as f64,
                                height as f64,
                                5.0,
                            ) {
                                this.re_render_image(height);
                            } else {
                                w.image_view.apply_zoom();
                                this.hq_redraw();
                            }
                        } else if backend.is_thumbnail() {
                            drop(backend);
                            this.update_thumbnail_backend();
                        } else {
                            w.image_view.apply_zoom();
                            this.hq_redraw();
                        }
                        this.current_height.set(height);
                        ControlFlow::Break
                    }
                ),
            )));
    }

    fn cancel_hq_redraw(&self) {
        if let Some(id) = self.hq_redraw_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {e}");
            }
        }
    }

    fn schedule_hq_redraw(&self) {
        self.hq_redraw_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_millis(DELAY_HQ_REDRAW),
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.hq_redraw_timeout_id.replace(None);
                        this.hq_redraw();
                        ControlFlow::Break
                    }
                ),
            )));
    }

    /// hq_redraw is different from image_view::redraw(QUALITY_HIGH) in case of
    /// zoomed documents. If we know for sure we will be handling non-documents
    /// we could use mage_view::redraw(QUALITY_HIGH). This is done is some places.
    pub fn hq_redraw(&self) {
        let zoom = self.widgets().image_view.zoom();
        let backend = self.backend.borrow();
        if backend.is_doc() && zoom.is_zoomed() {
            self.render_doc_zoom();
        } else {
            self.widgets().image_view.redraw(QUALITY_HIGH);
        }
    }

    /// Render a document in high detail through the backend::image_zoom operation
    /// which is only implemented on the document backend.
    pub fn render_doc_zoom(&self) {
        let w = self.widgets();
        if let Some(current) = w.file_view.current() {
            let params = ImageParams {
                sender: &w.sender,
                page_mode: &self.page_mode.get(),
                allocation_height: w.image_view.height(),
            };
            let backend = self.backend.borrow();
            if let Some(surface) = backend.image_zoom(
                &current,
                &params,
                w.image_view.image_size().1 as f32,
                w.image_view.clip(),
                w.image_view.zoom(),
            ) {
                w.image_view.set_zoomed_surface(surface);
            } else {
                w.image_view.redraw(QUALITY_HIGH); // Fallback if image_zoom fails or not implemented
            }
        }
    }

    // 4. Presentation needs to change
    //
    // caused by
    // - hover in case of thumbnail page
    //
    // actions:
    // - redraw in view
    //
    // 5. Content changes
    //
    // caused by
    // - change of image / backend
    // - rotate
    //
    // actions:
    // - (re)render content
}
