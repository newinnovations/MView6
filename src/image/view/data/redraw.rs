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
            data::{ZoomedImage, QUALITY_LOW},
            Zoom, QUALITY_HIGH,
        },
        ImageData,
    },
    rect::{RectD, SizeD},
    render_thread::model::RenderCommand,
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
    ImageChanged = 2,
    OverlayUpdated = 3,
    RotationChanged = 4,
    CanvasResized = 5,
    ZoomSettingChanged = 6,
    InteractiveDrag = 7,
    InteractiveZoom = 8,
    ImagePost = 9,
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
            2 => RedrawReason::ImageChanged,
            3 => RedrawReason::OverlayUpdated,
            4 => RedrawReason::RotationChanged,
            5 => RedrawReason::CanvasResized,
            6 => RedrawReason::ZoomSettingChanged,
            7 => RedrawReason::InteractiveDrag,
            8 => RedrawReason::InteractiveZoom,
            9 => RedrawReason::ImagePost,
            _ => RedrawReason::Unknown,
        }
    }
}

impl ImageViewData {
    fn redraw_quality(&mut self, quality: Filter, reason: RedrawReason) {
        println!("-- redraw  reason={reason:?}");
        self.quality = quality;
        if let Some(view) = &self.view {
            if quality == QUALITY_HIGH && reason != RedrawReason::OverlayUpdated {
                if let ImageData::Doc(pm, _s) = &self.image.image_data {
                    let a = view.allocation();
                    let viewport = RectD::new(0.0, 0.0, a.width() as f64, a.height() as f64);
                    self.rb_send(RenderCommand::RenderDoc(
                        self.image.reference.clone(),
                        self.image.id(),
                        *pm,
                        self.zoom.clone(),
                        viewport,
                    ));
                    if reason == RedrawReason::ImagePost {
                        return; // postpone actual redraw, because nothing to show
                                // TO CONSIDER
                                // actually with new images that are rendered by the bot
                                // we should postpone all redraws until we get an OverlayUpdated
                                // (which we may not get because, the images might already
                                //  have been updated for something else)
                    }
                } else if let ImageData::Svg(tree) = &self.image.image_data {
                    let a = view.allocation();
                    let viewport = RectD::new(0.0, 0.0, a.width() as f64, a.height() as f64);
                    self.rb_send(RenderCommand::RenderSvg(
                        self.image.id(),
                        self.zoom.clone(),
                        viewport,
                        tree.clone(),
                    ));
                    if reason == RedrawReason::ImagePost {
                        return;
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

    // pub fn clip(&self, a: Rectangle) -> RectD {
    //     if let Ok(matrix) = self.zoom.transform_matrix().try_invert() {
    //         let (x1, y1) = matrix.transform_point(0.0, 0.0);
    //         let (x2, y2) = matrix.transform_point(a.width() as f64, a.height() as f64);
    //         dbg!(x1, y1);
    //         dbg!(x2, y2);
    //         RectD {
    //             x0: x1.min(x2),
    //             y0: y1.min(y2),
    //             x1: x1.max(x2),
    //             y1: y1.max(y2),
    //         }
    //     } else {
    //         RectD {
    //             ..Default::default()
    //         }
    //     }
    // }

    // pub fn clip(&self, a: RectD) -> RectD {
    //     let origin = self.zoom.origin();
    //     let scale = 1.0 / self.zoom.zoom_factor();
    //     let top_left = VectorD::new(-origin.x(), -origin.y()).scale(scale);
    //     let bottom_right =
    //         VectorD::new(a.width() - origin.x(), a.height() - origin.y()).scale(scale);
    //     // dbg!(top_left, bottom_right);
    //     RectD {
    //         x0: top_left.x().min(bottom_right.x()),
    //         y0: top_left.y().min(bottom_right.y()),
    //         x1: top_left.x().max(bottom_right.x()),
    //         y1: top_left.y().max(bottom_right.y()),
    //     }
    // }

    pub fn hq_render_reply(&mut self, image_id: u32, surface_data: SurfaceData, orig_zoom: Zoom) {
        if self.image.id() != image_id {
            println!(
                "Got hq render for different image {} != {image_id}",
                self.image.id()
            );
            return;
        }
        if self.zoom != orig_zoom {
            println!(
                "Got hq render for different zoom {:?} != {orig_zoom:?}",
                self.zoom
            );
            return;
        }
        if let Ok(surface) = surface_data.surface() {
            let size = SizeD::new(surface.width() as f64, surface.height() as f64);
            let zoom = orig_zoom.new_unscaled(size);
            self.zoom_overlay = Some(ZoomedImage::new(surface, zoom.origin(), orig_zoom));
            self.redraw(RedrawReason::OverlayUpdated);
        }
    }
}
