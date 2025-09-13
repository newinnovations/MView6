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
pub mod zoom;

use cairo::{Filter, ImageSurface};
use glib::SourceId;
use gtk4::prelude::WidgetExt;

use crate::{
    backends::thumbnail::model::Annotations,
    content::{Content, ContentData},
    image::{Image, RenderedImage},
    rect::{PointD, RectD},
    render_thread::{model::RenderCommand, RenderThreadSender},
};

use super::{ImageView, Zoom, ZoomMode};

pub const QUALITY_HIGH: Filter = Filter::Bilinear;
pub const QUALITY_LOW: Filter = Filter::Fast;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparencyMode {
    #[default]
    NotSpecified,
    Checkerboard,
    White,
    Black,
}

impl From<&str> for TransparencyMode {
    fn from(value: &str) -> Self {
        match value {
            "checkerboard" => Self::Checkerboard,
            "white" => Self::White,
            "black" => Self::Black,
            _ => Self::NotSpecified,
        }
    }
}

impl From<TransparencyMode> for &str {
    fn from(value: TransparencyMode) -> Self {
        match value {
            TransparencyMode::NotSpecified => "",
            TransparencyMode::Checkerboard => "checkerboard",
            TransparencyMode::White => "white",
            TransparencyMode::Black => "black",
        }
    }
}

pub struct ImageViewData {
    pub content: Content,
    pub zoom: Zoom,
    pub zoom_mode: ZoomMode,
    pub zoom_overlay: Option<RenderedImage>,
    pub checkerboard: Option<ImageSurface>,
    pub transparency_mode: TransparencyMode,
    pub view: Option<ImageView>,
    pub mouse_position: PointD,
    pub drag: Option<PointD>,
    pub quality: Filter,
    pub annotations: Option<Annotations>,
    pub hover: Option<i32>,
    pub rb_sender: Option<RenderThreadSender>,
    hq_redraw_timeout_id: Option<SourceId>,
}

impl Default for ImageViewData {
    fn default() -> Self {
        Self {
            content: Content::default(),
            zoom: Zoom::default(),
            zoom_mode: ZoomMode::NotSpecified,
            zoom_overlay: None,
            checkerboard: None,
            transparency_mode: TransparencyMode::Checkerboard,
            view: None,
            mouse_position: PointD::default(),
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
            let size = self.content.size();
            let zoom_mode = if size.width() < 0.1 || size.height() < 0.1 {
                ZoomMode::NoZoom
            } else if self.content.zoom_mode == ZoomMode::NotSpecified {
                if self.zoom_mode == ZoomMode::NotSpecified {
                    ZoomMode::NoZoom
                } else {
                    self.zoom_mode
                }
            } else {
                self.content.zoom_mode
            };
            self.zoom.apply_zoom(zoom_mode, size, viewport);
        }
    }

    pub fn update_zoom(&mut self, new_zoom: f64, anchor: PointD) {
        self.zoom.update_zoom(new_zoom, anchor);
        if self.drag.is_some() {
            self.drag = Some(anchor - self.zoom.origin());
        }
    }

    pub fn rb_send(&self, command: RenderCommand) {
        if let Some(sender) = &self.rb_sender {
            sender.send_blocking(command);
        }
    }

    pub fn image(&'_ self) -> Image<'_> {
        if let Some(rendered) = &self.zoom_overlay {
            Image::Rendered(rendered)
        } else {
            match &self.content.data {
                ContentData::Single(single) => Image::Single(single),
                ContentData::Dual(dual) => Image::Dual(dual),
                ContentData::Animation(animation) => Image::Animation(animation),
                _ => Image::None,
            }
        }
    }
}
