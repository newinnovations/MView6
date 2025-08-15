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

use std::{
    cell::{Cell, RefCell},
    sync::OnceLock,
    time::SystemTime,
};

use crate::{
    category::Category,
    image::{
        colors::{CairoColorExt, Color},
        draw::transparency_background,
        view::{svg::render_svg, zoom::ZOOM_MULTIPLIER},
        Image, ImageData,
    },
    rect::{RectD, SizeD},
    util::remove_source_id,
};
use cairo::{Context, Extend, FillRule, SurfacePattern};
use gio::prelude::StaticType;
use glib::{clone, object::ObjectExt, subclass::Signal, ControlFlow, Propagation, SourceId};
use gtk4::{
    prelude::{DrawingAreaExtManual, GestureSingleExt, WidgetExt},
    subclass::prelude::*,
    EventControllerMotion, EventControllerScroll, EventControllerScrollFlags,
};

use super::{
    data::{ImageViewData, QUALITY_HIGH, QUALITY_LOW},
    ImageView, ViewCursor,
};

pub const SIGNAL_CANVAS_RESIZED: &str = "event-canvas-resized";
pub const SIGNAL_HQ_REDRAW: &str = "event-hq-redraw";

#[derive(Default)]
pub struct ImageViewImp {
    pub(super) data: RefCell<ImageViewData>,
    animation_timeout_id: RefCell<Option<SourceId>>,
    window_size: Cell<(i32, i32)>,
}

#[glib::object_subclass]
impl ObjectSubclass for ImageViewImp {
    const NAME: &'static str = "ImageWindow";
    type Type = ImageView;
    type ParentType = gtk4::DrawingArea;
}

impl ImageViewImp {
    pub fn cancel_animation(&self) {
        if let Some(id) = self.animation_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {e}");
            }
        }
    }

    pub fn schedule_animation(&self, image: &Image, ts_previous_cb: SystemTime) {
        if image.is_animation() {
            if let Some(interval) = image.animation_delay_time(ts_previous_cb) {
                // dbg!(interval);
                let current = self
                    .animation_timeout_id
                    .replace(Some(glib::timeout_add_local(
                        interval,
                        clone!(
                            #[weak(rename_to = this)]
                            self,
                            #[upgrade_or]
                            ControlFlow::Break,
                            move || {
                                this.animation_cb();
                                ControlFlow::Break
                            }
                        ),
                    )));
                assert!(current.is_none())
            }
        }
    }

    fn animation_cb(&self) {
        let start = SystemTime::now();
        self.animation_timeout_id.replace(None);
        let mut p = self.data.borrow_mut();
        if p.image.animation_advance(SystemTime::now()) {
            self.schedule_animation(&p.image, start);
            p.redraw(QUALITY_LOW);
        }
    }

    fn draw(&self, context: &Context) {
        let p = self.data.borrow();
        let z = &p.zoom;

        context.set_fill_rule(FillRule::EvenOdd);

        let viewport = clip_extents_to_rect(context);
        let intersect = z.intersection(&viewport);

        let (matrix, size, alpha) = if let Some((surface, zoom)) = &p.zoom_overlay {
            let size = SizeD::new(surface.width() as f64, surface.height() as f64);
            // let zoom = z.new_delta(parent_zoom, my_zoom);
            (zoom.transform_matrix(), size, false)
        } else if let ImageData::Svg(_tree) = &p.image.image_data {
            let size = z.pixmap_size(&intersect);
            (z.unscaled_transform_matrix(size), size, true)
        } else {
            (z.transform_matrix(), p.image.size(), p.image.has_alpha())
        };

        // Create black border around image
        context.color(Color::Black);
        // With FillRule::EvenOdd:
        // * Areas covered by an odd number of shapes get filled
        // * Areas covered by an even number of shapes don't get filled
        // * The outer rectangle covers the entire area (1 = odd, so filled)
        context.rectangle(
            viewport.x0,
            viewport.y0,
            viewport.width(),
            viewport.height(),
        );
        // * The inner rectangle overlaps part of the outer rectangle (1+1 = 2 = even, so not filled)
        // * make the black other area one pixel bigger at every side to avaid gaps
        //   around images in case of rounding errors
        context.rectangle(
            intersect.x0 + 1.0,
            intersect.y0 + 1.0,
            intersect.width() - 2.0,
            intersect.height() - 2.0,
        );
        // Result: black background with a unpainted "hole" in the middle
        let _ = context.fill();

        if alpha {
            if let Some(transparency_background) = &p.transparency_background {
                // Create a checkerboard pattern
                let pattern = SurfacePattern::create(transparency_background);
                pattern.set_extend(Extend::Repeat);
                let _ = context.set_source(&pattern);
                // make the checkerboard one pixel smaller at every side to not extend the
                // images in case of rounding errors
            }
        } else {
            context.color(Color::White);
        }
        context.rectangle(
            intersect.x0 + 1.0,
            intersect.y0 + 1.0,
            intersect.width() - 2.0,
            intersect.height() - 2.0,
        );
        let _ = context.fill();

        // Viewport offset is handled in the transformation matrix so drawing here happens
        // at the virtual origin (0.0, 0.0)
        context.transform(matrix);

        context.rectangle(0.0, 0.0, size.width(), size.height());
        if let Some((surface, _)) = &p.zoom_overlay {
            let _ = context.set_source_surface(surface, 0.0, 0.0);
            let _ = context.fill();
        } else if let ImageData::Svg(tree) = &p.image.image_data {
            render_svg(context, &p.zoom, &viewport, tree);
        } else {
            if let ImageData::Single(surface) = &p.image.image_data {
                let _ = context.set_source_surface(surface, 0.0, 0.0);
            } else if let ImageData::Dual(surface_left, surface_right) = &p.image.image_data {
                let (off_x_left, off_y_left, off_x_right, off_y_right) =
                    p.image.image_data.offset();
                let _ = context.set_source_surface(surface_left, off_x_left, off_y_left);
                context.source().set_filter(p.quality);
                let _ = context.fill();
                context.rectangle(0.0, 0.0, size.width(), size.height());
                let _ = context.set_source_surface(surface_right, off_x_right, off_y_right);
            }
            context.source().set_filter(p.quality);
            let _ = context.fill();
            self.draw_annotations(context);
        }
    }

    fn draw_annotations(&self, context: &Context) {
        let p = self.data.borrow();
        if let Some(annotations) = &p.annotations {
            let hover = annotations.get(p.hover);
            if let Some(hover) = hover {
                context.set_source_rgba(1.0, 1.0, 1.0, 0.1);
                context.rectangle(
                    hover.position.x,
                    hover.position.y,
                    hover.position.width,
                    hover.position.height,
                );
                let _ = context.fill_preserve();
                context.set_source_rgb(0.7, 0.7, 0.0);
                context.set_line_width(3.0);
                let _ = context.stroke();
            }

            for annotation in &annotations.annotations {
                match annotation.category {
                    Category::Favorite => context.set_source_rgb(0.0, 1.0, 0.0),
                    Category::Trash => context.set_source_rgb(1.0, 1.0, 0.0),
                    _ => continue,
                };
                context.arc(
                    annotation.position.x + annotation.position.width,
                    annotation.position.y + annotation.position.height,
                    if hover == Some(annotation) { 5.0 } else { 2.0 },
                    0.0,
                    2.0 * std::f64::consts::PI,
                );
                let _ = context.fill_preserve();
                context.set_line_width(2.0);
                let _ = context.stroke();
            }
        }
    }

    fn button_press_event(&self, position: (f64, f64)) {
        let mut p = self.data.borrow_mut();
        if p.drag.is_none() && p.image.is_movable() {
            let (position_x, position_y) = position;
            if let Some((_, zoom)) = &p.zoom_overlay {
                p.drag = Some((
                    position_x - p.zoom.offset_x(),
                    position_y - p.zoom.offset_y(),
                    position_x - zoom.offset_x(),
                    position_y - zoom.offset_y(),
                ));
            } else {
                p.drag = Some((
                    position_x - p.zoom.offset_x(),
                    position_y - p.zoom.offset_y(),
                    0.0,
                    0.0,
                ));
            }
            self.obj().set_view_cursor(ViewCursor::Drag);
        }
    }

    fn button_release_event(&self) {
        let mut p = self.data.borrow_mut();
        if p.drag.is_some() {
            p.drag = None;
            self.obj().set_view_cursor(ViewCursor::Normal);
            // p.redraw(QUALITY_HIGH);
        }
    }

    fn motion_notify_event(&self, x: f64, y: f64) {
        let mut p = self.data.borrow_mut();
        p.mouse_position = (x, y);
        if let Some(annotations) = &p.annotations {
            let index = annotations.index_at(x - p.zoom.offset_x(), y - p.zoom.offset_y());
            if index != p.hover {
                p.hover = index;
                p.redraw(QUALITY_HIGH); // hq_redraw not needed, because annotation only apply to thumbnail sheets
            }
        }
        if let Some((drag_x, drag_y, drag_ovl_x, drag_ovl_y)) = p.drag {
            p.zoom.set_offset(x - drag_x, y - drag_y);
            if let Some((_, zoom)) = &mut p.zoom_overlay {
                zoom.set_offset(x - drag_ovl_x, y - drag_ovl_y);
            }
            drop(p);
            self.obj().emit_by_name::<()>(SIGNAL_HQ_REDRAW, &[&true]);
        }
    }

    fn motion_leave_event(&self) {
        let mut p = self.data.borrow_mut();
        if p.hover.is_some() {
            p.hover = None;
            drop(p);
            self.hq_redraw(true);
        }
    }

    fn scroll_event(&self, dy: f64) -> Propagation {
        // self.cancel_hq_redraw();
        let mut p = self.data.borrow_mut();
        let mouse_position = p.mouse_position;
        if p.image.is_movable() {
            let zoom = if dy < -0.01 {
                p.zoom.zoom_factor() * ZOOM_MULTIPLIER
            } else if dy > 0.01 {
                p.zoom.zoom_factor() / ZOOM_MULTIPLIER
            } else {
                p.zoom.zoom_factor()
            };
            p.update_zoom(zoom, mouse_position);
            drop(p);
            self.hq_redraw(true);
        }
        Propagation::Stop
    }

    pub fn mouse_position(&self) -> (f64, f64) {
        self.data.borrow().mouse_position
    }

    pub fn hq_redraw(&self, delayed: bool) {
        self.obj().emit_by_name::<()>(SIGNAL_HQ_REDRAW, &[&delayed]);
    }
}

impl ObjectImpl for ImageViewImp {
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder(SIGNAL_CANVAS_RESIZED)
                    .param_types([i32::static_type(), i32::static_type()])
                    .build(),
                Signal::builder(SIGNAL_HQ_REDRAW)
                    .param_types([bool::static_type()])
                    .build(),
            ]
        })
    }

    fn constructed(&self) {
        self.parent_constructed();
        let view = self.obj();
        view.set_can_focus(true);
        view.set_hexpand(true);
        view.set_vexpand(true);

        self.data.borrow_mut().view = Some(view.clone());

        let motion_controller = EventControllerMotion::new();
        motion_controller.connect_motion(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, x, y| this.motion_notify_event(x, y)
        ));

        motion_controller.connect_leave(clone!(
            #[weak(rename_to = this)]
            self,
            move |_| this.motion_leave_event()
        ));

        let scroll_controller = EventControllerScroll::new(EventControllerScrollFlags::VERTICAL);
        scroll_controller.connect_scroll(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            Propagation::Stop,
            move |_, _dx, dy| this.scroll_event(dy)
        ));

        let gesture_click = gtk4::GestureClick::new();
        gesture_click.set_button(1);
        gesture_click.connect_pressed(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _n_press, x, y| this.button_press_event((x, y))
        ));
        gesture_click.connect_released(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _n_press, _x, _y| this.button_release_event()
        ));

        view.add_controller(motion_controller);
        view.add_controller(scroll_controller);
        view.add_controller(gesture_click);
    }
}

impl WidgetImpl for ImageViewImp {
    fn realize(&self) {
        self.parent_realize();

        let mut p = self.data.borrow_mut();
        p.transparency_background = transparency_background().ok();

        self.obj().set_draw_func(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, context, _width, _height| this.draw(context)
        ));
    }
}

impl DrawingAreaImpl for ImageViewImp {
    fn resize(&self, width: i32, height: i32) {
        let current_size = self.window_size.get();
        if current_size != (width, height) {
            // println!("view was resized to {width} {height}");
            self.window_size.set((width, height));
            self.obj()
                .emit_by_name::<()>(SIGNAL_CANVAS_RESIZED, &[&width, &height]);
        }
    }
}

/// Utility to convert clip_extents to rectangle
pub fn clip_extents_to_rect(context: &Context) -> RectD {
    if let Ok((x1, y1, x2, y2)) = context.clip_extents() {
        RectD::new(x1, y1, x2, y2)
    } else {
        eprintln!("Could not determine context.clip_extents()");
        Default::default() // Should not happen
    }
}
