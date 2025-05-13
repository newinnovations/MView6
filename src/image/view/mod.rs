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

mod data;
mod imp;

use std::time::SystemTime;

use data::QUALITY_HIGH;
use gdk_pixbuf::Pixbuf;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::{
    glib,
    prelude::WidgetExt,
};

use super::Image;
pub use imp::SIGNAL_VIEW_RESIZED;

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk4::DrawingArea, gtk4::Widget, @implements gtk4::Buildable;
}

impl Default for ImageView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomMode {
    #[default]
    NotSpecified,
    NoZoom,
    Fit,
    Fill,
    Max,
}

pub enum ViewCursor {
    Normal,
    Hidden,
    Drag,
}

impl ImageView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_image(&self, image: Image) {
        self.set_image_pre(image);
        self.set_image_post();
    }

    pub fn set_image_pre(&self, image: Image) {
        let mut p = self.imp().data.borrow_mut();
        self.imp().cancel_animation();
        p.image = image;
        // p.image.crop_to_max_size();
        p.rotation = 0;
    }

    pub fn set_image_post(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.create_surface();
        self.imp().schedule_animation(&p.image, SystemTime::now());
        p.apply_zoom();
    }

    pub fn image_modified(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.create_surface();
        p.redraw(QUALITY_HIGH);
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        let p = self.imp().data.borrow();
        p.zoom_mode
    }

    pub fn set_zoom_mode(&self, mode: ZoomMode) {
        let mut p = self.imp().data.borrow_mut();
        p.zoom_mode = mode;
        p.apply_zoom();
    }

    pub fn offset(&self) -> (f64, f64) {
        let p = self.imp().data.borrow();
        (p.xofs, p.yofs)
    }

    pub fn set_view_cursor(&self, view_cursor: ViewCursor) {
        match view_cursor {
            ViewCursor::Normal => self.set_cursor_from_name(Some("default")),
            ViewCursor::Hidden => self.set_cursor_from_name(Some("none")),
            ViewCursor::Drag => self.set_cursor_from_name(Some("move")),
        };
    }

    // Operations on image

    pub fn image_id(&self) -> u32 {
        self.imp().data.borrow().image.id()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        let p = self.imp().data.borrow();
        p.image.draw_pixbuf(pixbuf, dest_x, dest_y);
    }

    pub fn rotate(&self, angle: i32) {
        let mut p = self.imp().data.borrow_mut();
        let center = p.center();
        p.rotation = (p.rotation + angle).rem_euclid(360);
        p.image.rotate(angle);
        p.create_surface();
        p.move_center_to(center);
        p.redraw(QUALITY_HIGH);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.imp().data.borrow().image.has_tag(tag)
    }
}
