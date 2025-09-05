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

use cairo::Filter;
use gio::subclass::prelude::ObjectSubclassIsExt;
use glib::{clone, ControlFlow};
use gtk4::prelude::WidgetExt;

use crate::{
    image::{
        provider::surface::SurfaceData,
        view::{
            data::{RenderedImage, QUALITY_LOW},
            Zoom, QUALITY_HIGH,
        },
    },
    rect::RectD,
    util::remove_source_id,
};

use super::ImageViewData;

const DELAY_HQ_REDRAW: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum RedrawReason {
    Unknown = -1,
    AnimationCallback = 0,
    AnnotationChanged = 1,
    CanvasResized = 2,
    ContentChanged = 3,
    ContentPost = 4,
    InteractiveDrag = 5,
    InteractiveZoom = 6,
    PageChanged = 7,
    RenderingUpdated = 8,
    RotationChanged = 9,
    TransparencyBackgroundChanged = 10,
    ZoomSettingChanged = 11,
}

impl RedrawReason {
    pub fn delayed(&self) -> bool {
        matches!(self, Self::InteractiveDrag | Self::InteractiveZoom)
    }

    pub fn quality(&self) -> Filter {
        if matches!(self, Self::AnimationCallback) {
            QUALITY_LOW
        } else {
            QUALITY_HIGH
        }
    }
}

impl From<RedrawReason> for i32 {
    fn from(reason: RedrawReason) -> i32 {
        reason as i32
    }
}

impl From<i32> for RedrawReason {
    fn from(value: i32) -> Self {
        match value {
            0 => RedrawReason::AnimationCallback,
            1 => RedrawReason::AnnotationChanged,
            2 => RedrawReason::CanvasResized,
            3 => RedrawReason::ContentChanged,
            4 => RedrawReason::ContentPost,
            5 => RedrawReason::InteractiveDrag,
            6 => RedrawReason::InteractiveZoom,
            7 => RedrawReason::PageChanged,
            8 => RedrawReason::RenderingUpdated,
            9 => RedrawReason::RotationChanged,
            10 => RedrawReason::TransparencyBackgroundChanged,
            11 => RedrawReason::ZoomSettingChanged,
            _ => RedrawReason::Unknown,
        }
    }
}

impl ImageViewData {
    fn redraw_quality(&mut self, quality: Filter, reason: RedrawReason) {
        println!("-- redraw  reason={reason:?}");
        self.quality = quality;
        if let Some(view) = &self.view {
            if quality == QUALITY_HIGH
                && reason != RedrawReason::RenderingUpdated
                && self.content.needs_render()
            {
                let a = view.allocation();
                let viewport = RectD::new(0.0, 0.0, a.width() as f64, a.height() as f64);
                if let Some(command) = self.content.render(self.zoom.clone(), viewport) {
                    self.rb_send(command);
                    if reason == RedrawReason::ContentPost
                        || reason == RedrawReason::PageChanged
                        || reason == RedrawReason::RotationChanged
                    {
                        return; // postpone actual redraw, because nothing to show
                                // TO CONSIDER
                                // actually with new images that are rendered by the bot
                                // we should postpone all redraws until we get an OverlayUpdated
                                // (which we may not get because, the images might already
                                //  have been updated for something else)
                    }
                }
            }
            view.queue_draw();
        }
    }

    fn cancel_hq_redraw(&mut self) {
        if let Some(id) = &self.hq_redraw_timeout_id {
            if let Err(e) = remove_source_id(id) {
                eprintln!("remove_source_id: {e}");
            }
            self.hq_redraw_timeout_id = None;
        }
    }

    fn schedule_hq_redraw(&mut self, reason: RedrawReason, delay: u64) {
        if let Some(view) = &self.view {
            let view = view.imp();
            self.hq_redraw_timeout_id = Some(glib::timeout_add_local(
                Duration::from_millis(delay),
                clone!(
                    #[weak]
                    view,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        let mut p = view.data.borrow_mut();
                        p.hq_redraw_timeout_id = None;
                        p.redraw_quality(QUALITY_HIGH, reason);
                        ControlFlow::Break
                    }
                ),
            ));
        }
    }

    /// This is the public function to trigger a redraw
    pub fn redraw(&mut self, reason: RedrawReason) {
        self.cancel_hq_redraw();
        if reason.delayed() {
            self.schedule_hq_redraw(reason, DELAY_HQ_REDRAW);
            self.redraw_quality(QUALITY_LOW, reason);
        } else {
            self.redraw_quality(reason.quality(), reason);
        }
    }

    pub fn hq_render_reply(
        &mut self,
        image_id: u32,
        surface_data: SurfaceData,
        zoom: Zoom,
        viewport: RectD,
    ) {
        if self.content.id() != image_id {
            println!(
                "Got hq render for different image {} != {image_id}",
                self.content.id()
            );
            return;
        }
        if self.zoom != zoom {
            println!(
                "Got hq render for different zoom {:?} != {zoom:?}",
                self.zoom
            );
            return;
        }
        if let Ok(surface) = surface_data.surface() {
            let rect = zoom.intersection_screen_coord(&viewport);
            self.zoom_overlay = Some(RenderedImage::new(surface, zoom.top_left(&rect), zoom));
            self.redraw(RedrawReason::RenderingUpdated);
        }
    }
}
