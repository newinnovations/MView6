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

use std::slice;

use cairo::{Filter, Format, ImageSurface};
use gdk_pixbuf::{
    ffi::{gdk_pixbuf_get_byte_length, gdk_pixbuf_read_pixels},
    Pixbuf,
};
use glib::translate::ToGlibPtr;
use gtk4::prelude::WidgetExt;

use crate::{
    backends::thumbnail::model::Annotations,
    image::{view::zoom::ImageZoom, Image, ImageData},
    profile::performance::Performance,
};

use super::{ImageView, ZoomMode};

pub const QUALITY_HIGH: Filter = Filter::Bilinear;
pub const QUALITY_LOW: Filter = Filter::Fast;

pub enum Surfaces {
    None,
    Single(ImageSurface),
    Dual(ImageSurface, ImageSurface, f64, f64, f64),
}

// impl Surfaces {
//     pub fn is_dual(&self) -> bool {
//         matches!(self, Surfaces::Dual(_, _, _, _, _))
//     }
// }

pub struct ImageViewData {
    pub image: Image,
    pub zoom_mode: ZoomMode,
    pub surface: Surfaces,
    pub zoom_surface: Option<ImageSurface>,
    pub transparency_background: Option<ImageSurface>,
    pub view: Option<ImageView>,
    pub mouse_position: (f64, f64),
    pub drag: Option<(f64, f64)>,
    pub quality: Filter,
    pub annotations: Option<Annotations>,
    pub hover: Option<i32>,
    pub zoom: ImageZoom,
}

impl Default for ImageViewData {
    fn default() -> Self {
        Self {
            image: Image::default(),
            zoom_mode: ZoomMode::NotSpecified,
            // rotation: 0,
            surface: Surfaces::None,
            zoom_surface: None,
            transparency_background: None,
            view: None,
            mouse_position: (0.0, 0.0),
            drag: None,
            quality: QUALITY_HIGH,
            annotations: Default::default(),
            hover: None,
            zoom: ImageZoom::default(),
        }
    }
}

// https://users.rust-lang.org/t/converting-a-bgra-u8-to-rgb-u8-n-for-images/67938
fn create_surface_single(p: &Pixbuf) -> Result<ImageSurface, cairo::Error> {
    let duration = Performance::start();

    let width = p.width() as usize;
    let height = p.height() as usize;
    let pixbuf_stride = p.rowstride() as usize;

    let surface_stride = 4 * width;
    let mut surface_data = vec![0_u8; height * surface_stride];

    unsafe {
        // gain access without the copy of pixbuf memory
        let pixbuf_data_raw = gdk_pixbuf_read_pixels(p.to_glib_none().0);
        let pixbuf_data_len = gdk_pixbuf_get_byte_length(p.to_glib_none().0);
        let pixbuf_data = slice::from_raw_parts(pixbuf_data_raw, pixbuf_data_len);

        if p.has_alpha() {
            for (src_row, dst_row) in pixbuf_data
                .chunks_exact(pixbuf_stride)
                .zip(surface_data.chunks_exact_mut(surface_stride))
            {
                for (src, dst) in src_row.chunks_exact(4).zip(dst_row.chunks_exact_mut(4)) {
                    dst[0] = src[2];
                    dst[1] = src[1];
                    dst[2] = src[0];
                    dst[3] = src[3];
                }
            }
        } else {
            for (src_row, dst_row) in pixbuf_data
                .chunks_exact(pixbuf_stride)
                .zip(surface_data.chunks_exact_mut(surface_stride))
            {
                for (src, dst) in src_row.chunks_exact(3).zip(dst_row.chunks_exact_mut(4)) {
                    dst[0] = src[2];
                    dst[1] = src[1];
                    dst[2] = src[0];
                    dst[3] = 255;
                }
            }
        }
    }

    let surface = ImageSurface::create_for_data(
        surface_data,
        Format::ARgb32,
        width as i32,
        height as i32,
        surface_stride as i32,
    );

    duration.elapsed("surface");

    surface
}

impl ImageViewData {
    pub(super) fn create_surface(&mut self) {
        if let ImageData::Single(pixbuf) = &self.image.image_data {
            if let Ok(surface) = create_surface_single(pixbuf) {
                self.surface = Surfaces::Single(surface);
            } else {
                self.surface = Surfaces::None;
            }
        } else if let ImageData::Dual(pixbuf1, pixbuf2) = &self.image.image_data {
            if let (Ok(surface1), Ok(surface2)) = (
                create_surface_single(pixbuf1),
                create_surface_single(pixbuf2),
            ) {
                let w1 = surface1.width() as f64;
                let h1 = surface1.height() as f64;
                let h2 = surface2.height() as f64;
                if h1 > h2 {
                    self.surface = Surfaces::Dual(surface1, surface2, w1, 0.0, (h1 - h2) / 2.0);
                } else {
                    self.surface = Surfaces::Dual(surface1, surface2, w1, (h2 - h1) / 2.0, 0.0);
                }
            } else {
                self.surface = Surfaces::None;
            }
        } else {
            self.surface = Surfaces::None;
        }
    }

    pub fn redraw(&mut self, quality: Filter) {
        if let Some(view) = &self.view {
            self.zoom_surface = None;
            self.quality = quality;
            view.queue_draw();
        }
    }

    pub fn apply_zoom(&mut self) {
        if let Some(view) = &self.view {
            let viewport = view.allocation();
            let image_size = self.image.size();
            let (image_width, image_height) = image_size;
            let zoom_mode = if image_width < 0.1 || image_height < 0.1 {
                ZoomMode::NoZoom
            } else if self.image.zoom_mode == ZoomMode::NotSpecified {
                if self.zoom_mode == ZoomMode::NotSpecified {
                    ZoomMode::NoZoom
                } else {
                    self.zoom_mode
                }
            } else {
                self.image.zoom_mode
            };
            self.zoom.apply_zoom(zoom_mode, image_size, viewport);
        }
    }

    pub fn update_zoom(&mut self, new_zoom: f64, anchor: (f64, f64)) {
        self.zoom.update_zoom(new_zoom, anchor);
        if self.drag.is_some() {
            let (anchor_x, anchor_y) = anchor;
            self.drag = Some((anchor_x - self.zoom.off_x(), anchor_y - self.zoom.off_y()))
        }
    }
}
