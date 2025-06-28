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

use cairo::{Filter, ImageSurface, Matrix};
use gdk_pixbuf::{
    ffi::{gdk_pixbuf_get_byte_length, gdk_pixbuf_read_pixels},
    Pixbuf,
};
use glib::translate::ToGlibPtr;
use gtk4::prelude::WidgetExt;

use crate::{
    backends::thumbnail::model::Annotations,
    image::{Image, ImageData},
    profile::performance::Performance,
};

use super::{ImageView, ZoomMode};

pub const MAX_ZOOM_FACTOR: f64 = 30.0;
pub const MIN_ZOOM_FACTOR: f64 = 0.02;
pub const ZOOM_MULTIPLIER: f64 = 1.05;
pub const QUALITY_HIGH: Filter = Filter::Bilinear;
pub const QUALITY_LOW: Filter = Filter::Fast;

pub enum Surfaces {
    None,
    Single(ImageSurface),
    Dual(ImageSurface, ImageSurface, f64, f64, f64),
}

impl Surfaces {
    pub fn is_dual(&self) -> bool {
        matches!(self, Surfaces::Dual(_, _, _, _, _))
    }
}

#[derive(Debug, Clone)]
pub struct ImageZoom {
    pub rotation: i32,
    screen_off_x: f64, // to center image on screen
    screen_off_y: f64,
    image_off_x: f64, // to correct for (0,0) origin with rotated images
    image_off_y: f64,
    pub zoom: f64,
}

impl Default for ImageZoom {
    fn default() -> Self {
        Self {
            rotation: 0,
            screen_off_x: 0.0,
            screen_off_y: 0.0,
            image_off_x: 0.0,
            image_off_y: 0.0,
            zoom: 1.0,
        }
    }
}

impl ImageZoom {
    pub fn state(&self) -> ZoomState {
        if self.zoom > 1.0 + 1.0e-6 {
            ZoomState::ZoomedIn
        } else if self.zoom < 1.0 - 1.0e-6 {
            ZoomState::ZoomedOut
        } else {
            ZoomState::NoZoom
        }
    }

    pub fn off_x(&self) -> f64 {
        self.screen_off_x + self.image_off_x
    }

    pub fn off_y(&self) -> f64 {
        self.screen_off_y + self.image_off_y
    }

    pub fn set_offset(&mut self, off_x: f64, off_y: f64) {
        self.screen_off_x = off_x - self.image_off_x;
        self.screen_off_y = off_y - self.image_off_y;
    }

    pub fn matrix(&self) -> Matrix {
        match self.rotation % 360 {
            90 => Matrix::new(0.0, self.zoom, -self.zoom, 0.0, self.off_x(), self.off_y()),
            180 => Matrix::new(-self.zoom, 0.0, 0.0, -self.zoom, self.off_x(), self.off_y()),
            270 => Matrix::new(0.0, -self.zoom, self.zoom, 0.0, self.off_x(), self.off_y()),
            _ => Matrix::new(self.zoom, 0.0, 0.0, self.zoom, self.off_x(), self.off_y()),
        }
    }

    pub fn clip_matrix(&self, width: i32, height: i32) -> Matrix {
        let screen_off_x = self.screen_off_x.max(0.0);
        let screen_off_y = self.screen_off_y.max(0.0);
        match self.rotation % 360 {
            90 => Matrix::new(
                0.0,
                1.0,
                -1.0,
                0.0,
                screen_off_x + height as f64,
                screen_off_y,
            ),
            180 => Matrix::new(
                -1.0,
                0.0,
                0.0,
                -1.0,
                screen_off_x + width as f64,
                screen_off_y + height as f64,
            ),
            270 => Matrix::new(
                0.0,
                -1.0,
                1.0,
                0.0,
                screen_off_x,
                screen_off_y + width as f64,
            ),
            _ => Matrix::new(1.0, 0.0, 0.0, 1.0, screen_off_x, screen_off_y),
        }
    }
}

pub struct ImageViewData {
    pub image: Image,
    pub zoom_mode: ZoomMode,
    pub surface: Surfaces,
    pub zoom_surface: Option<ImageSurface>,
    pub transparency_background: Option<ImageSurface>,
    pub view: Option<ImageView>,
    pub mouse_position: (f64, f64),
    pub drag: Option<(f64, f64)>,
    pub quality: cairo::Filter,
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

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomState {
    NoZoom,
    ZoomedIn,
    ZoomedOut,
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
        cairo::Format::ARgb32,
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

    fn compute_scaled_size(&self, zoom: f64) -> (f64, f64) {
        let (width, height) = self.image.size();
        (width * zoom, height * zoom)
    }

    pub fn image_coords(&self) -> (f64, f64, f64, f64) {
        let (scaled_width, scaled_height) = self.compute_scaled_size(self.zoom.zoom);
        (
            self.zoom.screen_off_x,
            self.zoom.screen_off_y,
            scaled_width,
            scaled_height,
        )
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
            let allocation = view.allocation();
            let allocation_width = allocation.width() as f64;
            let allocation_height = allocation.height() as f64;
            let (width, height) = self.image.size();

            let (size_x, size_y) = match self.zoom.rotation {
                90 | 270 => (height, width),
                _ => (width, height),
            };

            let zoom_mode = if size_x < 0.1 || size_y < 0.1 {
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

            let zoom = if zoom_mode == ZoomMode::NoZoom {
                1.0
            } else {
                let zoom1 = allocation_width / size_x;
                let zoom2 = allocation_height / size_y;
                if zoom_mode == ZoomMode::Max {
                    if zoom1 > zoom2 {
                        zoom1
                    } else {
                        zoom2
                    }
                } else if zoom_mode == ZoomMode::Fit
                    && allocation_width > size_x
                    && allocation_height > size_y
                {
                    1.0
                } else if zoom1 > zoom2 {
                    zoom2
                } else {
                    zoom1
                }
            };

            self.zoom.zoom = zoom.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);

            let size_x_zoomed = self.zoom.zoom * size_x;
            let size_y_zoomed = self.zoom.zoom * size_y;

            let (image_off_x, image_off_y) = match self.zoom.rotation {
                90 => (size_x_zoomed, 0.0),
                180 => (size_x_zoomed, size_y_zoomed),
                270 => (0.0, size_y_zoomed),
                _ => (0.0, 0.0),
            };

            self.zoom.image_off_x = image_off_x;
            self.zoom.image_off_y = image_off_y;
            self.zoom.screen_off_x = ((allocation_width - size_x_zoomed) / 2.0).round();
            self.zoom.screen_off_y = ((allocation_height - size_y_zoomed) / 2.0).round();
        }
    }

    pub fn update_zoom(&mut self, zoom: f64, anchor: (f64, f64)) {
        let old_zoom = self.zoom.zoom;
        let new_zoom = zoom.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
        if new_zoom == old_zoom {
            return;
        }
        let (anchor_x, anchor_y) = anchor;
        let view_cx = (anchor_x - self.zoom.off_x()) / old_zoom;
        let view_cy = (anchor_y - self.zoom.off_y()) / old_zoom;

        self.zoom.image_off_x = self.zoom.image_off_x * new_zoom / old_zoom;
        self.zoom.image_off_y = self.zoom.image_off_y * new_zoom / old_zoom;

        self.zoom
            .set_offset(anchor_x - view_cx * new_zoom, anchor_y - view_cy * new_zoom);
        self.zoom.zoom = new_zoom;
        if self.drag.is_some() {
            self.drag = Some((anchor_x - self.zoom.off_x(), anchor_y - self.zoom.off_y()))
        }
    }
}
