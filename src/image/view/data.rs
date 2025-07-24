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

use cairo::{Filter, ImageSurface};
use gtk4::prelude::WidgetExt;

use crate::{
    backends::thumbnail::model::Annotations,
    image::{view::zoom::Zoom, Image},
};

use super::{ImageView, ZoomMode};

pub const QUALITY_HIGH: Filter = Filter::Bilinear;
pub const QUALITY_LOW: Filter = Filter::Fast;

pub struct ImageViewData {
    pub image: Image,
    pub zoom: Zoom,
    pub zoom_mode: ZoomMode,
    pub zoom_overlay: Option<ImageSurface>,
    pub transparency_background: Option<ImageSurface>,
    pub view: Option<ImageView>,
    pub mouse_position: (f64, f64),
    pub drag: Option<(f64, f64)>,
    pub quality: Filter,
    pub annotations: Option<Annotations>,
    pub hover: Option<i32>,
}

impl Default for ImageViewData {
    fn default() -> Self {
        Self {
            image: Image::default(),
            zoom: Zoom::default(),
            zoom_mode: ZoomMode::NotSpecified,
            zoom_overlay: None,
            transparency_background: None,
            view: None,
            mouse_position: (0.0, 0.0),
            drag: None,
            quality: QUALITY_HIGH,
            annotations: Default::default(),
            hover: None,
        }
    }
}

impl ImageViewData {
    pub fn redraw(&mut self, quality: Filter) {
        if let Some(view) = &self.view {
            self.zoom_overlay = None;
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
