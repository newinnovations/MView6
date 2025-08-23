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

pub mod redraw;

use cairo::{Filter, ImageSurface, Matrix};
use glib::SourceId;
use gtk4::prelude::WidgetExt;

use crate::{
    backends::thumbnail::model::Annotations,
    image::{view::zoom::Zoom, Image},
    rect::{RectD, SizeD, VectorD},
    render_thread::{model::RenderCommand, RenderThreadSender},
};

use super::{ImageView, ZoomMode};

pub const QUALITY_HIGH: Filter = Filter::Bilinear;
pub const QUALITY_LOW: Filter = Filter::Fast;

#[derive(Debug, Clone)]
pub struct ZoomedImage {
    surface: ImageSurface,
    origin: VectorD,
    orig_image_zoom: Zoom,
}

impl ZoomedImage {
    pub fn new(surface: ImageSurface, origin: VectorD, orig_image_zoom: Zoom) -> Self {
        Self {
            surface,
            origin,
            orig_image_zoom,
        }
    }

    pub fn surface(&self) -> &ImageSurface {
        &self.surface
    }

    pub fn size(&self) -> SizeD {
        SizeD::new(self.surface.width() as f64, self.surface.height() as f64)
    }

    /// Creates a Cairo transformation matrix for displaying this zoomed image
    ///
    /// It corrects for the situation that the current zoom (scale and position) may have
    /// changed from the original zoom for which this rendering was made. And that until
    /// we have an updated rendering for the current zoom, we must scale and transpose this one.
    pub fn transform_matrix(&self, current_image_zoom: &Zoom) -> Matrix {
        let scale = current_image_zoom.scale() / self.orig_image_zoom.scale();
        let new_origin = current_image_zoom.origin() + self.origin.scale(scale)
            - self.orig_image_zoom.origin().scale(scale);
        let mut zoom = self.orig_image_zoom.clone();
        zoom.set_origin(new_origin);
        zoom.set_zoom_factor(scale);
        zoom.transform_matrix()
    }
}

pub struct ImageViewData {
    pub image: Image,
    pub zoom: Zoom,
    pub zoom_mode: ZoomMode,
    pub zoom_overlay: Option<ZoomedImage>,
    pub transparency_background: Option<ImageSurface>,
    pub view: Option<ImageView>,
    pub mouse_position: (f64, f64), // FIXME: change to VectorD
    pub drag: Option<(f64, f64)>,   // FIXME: change to VectorD
    pub quality: Filter,
    pub annotations: Option<Annotations>,
    pub hover: Option<i32>,
    pub rb_sender: Option<RenderThreadSender>,
    hq_redraw_timeout_id: Option<SourceId>,
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
            rb_sender: None,
            hq_redraw_timeout_id: None,
        }
    }
}

impl ImageViewData {
    pub fn apply_zoom(&mut self) {
        if let Some(view) = &self.view {
            let allocation = view.allocation();
            // * allocation is relative to the parent window, allocation.x() and
            //   allocation.y() might not be 0 depending on the presence of other
            //   widgets in the parent window (fileview, borders, etc)
            // * viewport is relatieve to the view, so origin is (0.0, 0.0)
            let viewport = RectD::new(
                0.0,
                0.0,
                allocation.width() as f64,
                allocation.height() as f64,
            );
            let size = self.image.size();
            let zoom_mode = if size.width() < 0.1 || size.height() < 0.1 {
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
            self.zoom.apply_zoom(zoom_mode, size, viewport);
        }
    }

    pub fn update_zoom(&mut self, new_zoom: f64, anchor: (f64, f64)) {
        self.zoom.update_zoom(new_zoom, anchor);
        if self.drag.is_some() {
            let (anchor_x, anchor_y) = anchor;
            self.drag = Some((
                anchor_x - self.zoom.offset_x(),
                anchor_y - self.zoom.offset_y(),
            ))
        }
    }

    pub fn rb_send(&self, command: RenderCommand) {
        if let Some(sender) = &self.rb_sender {
            sender.send_blocking(command);
        }
    }
}
