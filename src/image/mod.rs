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

pub mod animation;
pub mod colors;
pub mod draw;
pub mod provider;
pub mod view;

use animation::Animation;
use cairo::ImageSurface;
use exif::Exif;
use gdk_pixbuf::Pixbuf;
use glib::translate::from_glib_full;
use gtk4::gdk::ffi::gdk_pixbuf_get_from_surface;
use rsvg::{prelude::HandleExt, Handle};
use std::{
    cmp::max,
    cmp::min,
    sync::atomic::{AtomicU32, Ordering},
};
use view::ZoomMode;

pub const MAX_IMAGE_SIZE: i32 = 32767;
static IMAGE_ID: AtomicU32 = AtomicU32::new(1);

fn get_image_id() -> u32 {
    IMAGE_ID.fetch_add(1, Ordering::SeqCst);
    IMAGE_ID.load(Ordering::SeqCst)
}

#[derive(Default)]
pub enum ImageData {
    #[default]
    None,
    Single(Pixbuf),
    Dual(Pixbuf, Pixbuf),
    Svg(Handle),
}

impl From<Option<Pixbuf>> for ImageData {
    fn from(value: Option<Pixbuf>) -> Self {
        match value {
            Some(pixbuf) => ImageData::Single(pixbuf),
            None => ImageData::None,
        }
    }
}

impl From<(Option<Pixbuf>, Option<Pixbuf>)> for ImageData {
    fn from(value: (Option<Pixbuf>, Option<Pixbuf>)) -> Self {
        match value {
            (Some(pixbuf), None) => ImageData::Single(pixbuf),
            (None, Some(pixbuf)) => ImageData::Single(pixbuf),
            (Some(pixbuf1), Some(pixbuf2)) => ImageData::Dual(pixbuf1, pixbuf2),
            (None, None) => ImageData::None,
        }
    }
}

#[derive(Default)]
pub struct Image {
    id: u32,
    pub image_data: ImageData,
    animation: Animation,
    pub exif: Option<Exif>,
    zoom_mode: ZoomMode,
    tag: Option<String>,
}

impl Image {
    pub fn new_surface(surface: &ImageSurface, zoom_mode: ZoomMode) -> Self {
        let pixbuf: Option<Pixbuf> = unsafe {
            from_glib_full(gdk_pixbuf_get_from_surface(
                surface.as_ref().to_raw_none(),
                0,
                0,
                surface.width(),
                surface.height(),
            ))
        };
        Image {
            id: get_image_id(),
            image_data: pixbuf.into(),
            animation: Animation::None,
            exif: None,
            zoom_mode,
            tag: None,
        }
    }

    pub fn new_pixbuf(pixbuf: Option<Pixbuf>, exif: Option<Exif>) -> Self {
        Image {
            id: get_image_id(),
            image_data: pixbuf.into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_dual_pixbuf(
        pixbuf_left: Option<Pixbuf>,
        pixbuf_right: Option<Pixbuf>,
        exif: Option<Exif>,
    ) -> Self {
        Image {
            id: get_image_id(),
            image_data: (pixbuf_left, pixbuf_right).into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_animation(animation: Animation) -> Self {
        let pixbuf = match &animation {
            Animation::None => None,
            Animation::Gdk(a) => Some(a.pixbuf()),
            Animation::WebPFile(a) => a.pixbuf_get(0),
            Animation::WebPMemory(a) => a.pixbuf_get(0),
        };
        Image {
            id: get_image_id(),
            image_data: pixbuf.into(),
            animation,
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_svg(svg: Handle, tag: Option<String>, zoom_mode: ZoomMode) -> Self {
        Image {
            id: get_image_id(),
            image_data: ImageData::Svg(svg),
            animation: Animation::None,
            exif: None,
            zoom_mode,
            tag,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn size(&self) -> (f64, f64) {
        match &self.image_data {
            ImageData::None => (0.0, 0.0),
            ImageData::Single(pixbuf) => (pixbuf.width() as f64, pixbuf.height() as f64),
            ImageData::Dual(pixbuf1, pixbuf2) => (
                (pixbuf1.width() + pixbuf2.width()) as f64,
                max(pixbuf1.height(), pixbuf2.height()) as f64,
            ),
            ImageData::Svg(handle) => handle.intrinsic_size_in_pixels().unwrap_or((64.0, 64.0)),
        }
    }

    pub fn has_alpha(&self) -> bool {
        match &self.image_data {
            ImageData::None => false,
            ImageData::Single(pixbuf) => pixbuf.has_alpha(),
            ImageData::Dual(pixbuf1, pixbuf2) => pixbuf1.has_alpha() || pixbuf2.has_alpha(),
            ImageData::Svg(_handle) => true,
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        match &self.tag {
            Some(t) => t.eq(tag),
            None => false,
        }
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        self.zoom_mode
    }

    pub fn is_movable(&self) -> bool {
        self.zoom_mode != ZoomMode::NoZoom
    }

    pub fn exif(&self) -> Option<&Exif> {
        self.exif.as_ref()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        if let ImageData::Single(my_pixbuf) = &self.image_data {
            pixbuf.copy_area(
                0,
                0,
                pixbuf.width(),
                pixbuf.height(),
                my_pixbuf,
                dest_x,
                dest_y,
            );
        }
    }

    pub fn crop_to_max_size(&mut self) {
        if let ImageData::Single(pixbuf) = &self.image_data {
            if pixbuf.width() > MAX_IMAGE_SIZE || pixbuf.height() > MAX_IMAGE_SIZE {
                let new_width = min(pixbuf.width(), MAX_IMAGE_SIZE);
                let new_height = min(pixbuf.height(), MAX_IMAGE_SIZE);
                let new_pixpuf = Pixbuf::new(
                    pixbuf.colorspace(),
                    pixbuf.has_alpha(),
                    pixbuf.bits_per_sample(),
                    new_width,
                    new_height,
                );
                if let Some(new_pixbuf) = &new_pixpuf {
                    pixbuf.copy_area(0, 0, new_width, new_height, new_pixbuf, 0, 0);
                }
                self.image_data = new_pixpuf.into();
                self.animation = Animation::None;
            }
        }
    }
}
