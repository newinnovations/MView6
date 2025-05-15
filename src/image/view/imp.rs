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

use std::{
    cell::RefCell,
    sync::OnceLock,
    time::{Duration, SystemTime},
};

use crate::image::{
    colors::{CairoColorExt, Color},
    draw::transparency_background,
    Image, ImageData,
};
use gio::prelude::StaticType;
use glib::{
    clone, ffi::g_source_remove, object::ObjectExt, result_from_gboolean, subclass::Signal,
    BoolError, ControlFlow, Propagation, SourceId,
};
use gtk4::{
    prelude::{DrawingAreaExtManual, GestureSingleExt, WidgetExt},
    subclass::prelude::*,
    EventControllerMotion, EventControllerScroll, EventControllerScrollFlags,
};
use rsvg::prelude::HandleExt;

use super::{
    data::{ImageViewData, Surfaces, ZoomState, QUALITY_HIGH, QUALITY_LOW, ZOOM_MULTIPLIER},
    ImageView, ViewCursor,
};

pub const SIGNAL_VIEW_RESIZED: &str = "view-resized";

#[derive(Default)]
pub struct ImageViewImp {
    pub(super) data: RefCell<ImageViewData>,
    animation_timeout_id: RefCell<Option<SourceId>>,
    hq_redraw_timeout_id: RefCell<Option<SourceId>>,
    resize_notify_timeout_id: RefCell<Option<SourceId>>,
}

#[glib::object_subclass]
impl ObjectSubclass for ImageViewImp {
    const NAME: &'static str = "ImageWindow";
    type Type = ImageView;
    type ParentType = gtk4::DrawingArea;
}

fn remove_source_id(id: SourceId) -> Result<(), BoolError> {
    unsafe { result_from_gboolean!(g_source_remove(id.as_raw()), "Failed to remove source") }
}

impl ImageViewImp {
    pub fn cancel_animation(&self) {
        if let Some(id) = self.animation_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {}", e);
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
            let rotation = p.rotation;
            p.image.rotate(rotation);
            p.create_surface();
            self.schedule_animation(&p.image, start);
            p.redraw(QUALITY_LOW);
        }
    }

    fn cancel_hq_redraw(&self) {
        if let Some(id) = self.hq_redraw_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {}", e);
            }
        }
    }

    fn schedule_hq_redraw(&self) {
        self.hq_redraw_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_millis(100),
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.hq_redraw_timeout_id.replace(None);
                        let mut p = this.data.borrow_mut();
                        p.redraw(QUALITY_HIGH);
                        ControlFlow::Break
                    }
                ),
            )));
    }

    fn cancel_resize_notify(&self) {
        if let Some(id) = self.resize_notify_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(id) {
                println!("remove_source_id: {}", e);
            }
        }
    }

    fn schedule_resize_notify(&self) {
        self.resize_notify_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_millis(300),
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.resize_notify_timeout_id.replace(None);

                        // let mut p = this.data.borrow_mut();
                        // println!(
                        //     "p.mouse_position {} {}",
                        //     p.mouse_position.0, p.mouse_position.1
                        // );
                        // match this.obj().get_window_relative_cursor_position() {
                        //     Ok(position) => {
                        //         p.mouse_position = position;
                        //         println!("position {} {}", position.0, position.1)
                        //     }
                        //     Err(err) => println!("error {}", err),
                        // }

                        // println!("notify of resize");
                        let obj = this.obj();
                        let allocation = obj.allocation();
                        obj.emit_by_name::<()>(
                            SIGNAL_VIEW_RESIZED,
                            &[&allocation.width(), &allocation.height()],
                        );
                        // let mut p = this.data.borrow_mut();
                        // p.redraw(QUALITY_HIGH);
                        ControlFlow::Break
                    }
                ),
            )));
    }

    fn draw(&self, context: &cairo::Context) {
        let p = self.data.borrow();

        let (xofs, yofs, scaled_width, scaled_height) = p.image_coords();

        /* Paint the background */
        let allocation = self.obj().allocation();
        context.rectangle(
            0.0,
            0.0,
            allocation.width() as f64,
            allocation.height() as f64,
        );

        // cr.set_source_rgba(0.0, 0.0, 0.0, 1.0);
        context.color(Color::Black);
        context.set_fill_rule(cairo::FillRule::EvenOdd);
        let _ = context.fill();

        if !p.surface.is_dual() && p.image.has_alpha() {
            if let Some(transparency_background) = &p.transparency_background {
                let _ = context.set_source_surface(transparency_background, xofs, yofs);
                context.source().set_extend(cairo::Extend::Repeat);
                context.rectangle(xofs, yofs, scaled_width, scaled_height);
                let _ = context.fill();
            }
        }

        /* Make sure the image is only drawn as large as needed.
         * This is especially necessary for SVGs where there might
         * be more image data available outside the image boundaries.
         */
        context.rectangle(xofs, yofs, scaled_width, scaled_height);
        context.clip();
        if let ImageData::Svg(handle) = &p.image.image_data {
            let viewport = rsvg::Rectangle::new(xofs, yofs, scaled_width, scaled_height);
            handle.render_document(context, &viewport).unwrap();
        } else {
            context.scale(p.zoom, p.zoom);
            if p.zoom_state() != ZoomState::NoZoom {
                context.source().set_filter(p.quality);
            }
            if let Surfaces::Single(surface) = &p.surface {
                let _ = context.set_source_surface(surface, xofs / p.zoom, yofs / p.zoom);
            } else if let Surfaces::Dual(surface1, surface2, w1, y1, y2) = &p.surface {
                let _ = context.set_source_surface(surface1, xofs / p.zoom, yofs / p.zoom + y1);
                let _ = context.paint();
                let _ =
                    context.set_source_surface(surface2, w1 + xofs / p.zoom, yofs / p.zoom + y2);
            }
            let _ = context.paint();
        }
    }

    fn button_press_event(&self, position: (f64, f64)) {
        let mut p = self.data.borrow_mut();
        if p.drag.is_none() && p.image.is_movable() {
            let (position_x, position_y) = position;
            p.drag = Some((position_x + p.xofs, position_y + p.yofs));
            self.obj().set_view_cursor(ViewCursor::Drag);
        }
    }

    fn button_release_event(&self) {
        let mut p = self.data.borrow_mut();
        if p.drag.is_some() {
            p.drag = None;
            self.obj().set_view_cursor(ViewCursor::Normal);
            p.redraw(QUALITY_HIGH);
        }
    }

    fn motion_notify_event(&self, position: (f64, f64)) {
        let mut p = self.data.borrow_mut();
        p.mouse_position = position;
        if let Some((drag_x, drag_y)) = p.drag {
            let (position_x, position_y) = position;
            p.xofs = drag_x - position_x;
            p.yofs = drag_y - position_y;
            p.redraw(QUALITY_LOW);
        }
    }

    fn scroll_event(&self, dy: f64) -> Propagation {
        self.cancel_hq_redraw();
        let mut p = self.data.borrow_mut();
        let mouse_position = p.mouse_position;
        if p.image.is_movable() {
            let zoom = if dy < -0.01 {
                p.zoom * ZOOM_MULTIPLIER
            } else if dy > 0.01 {
                p.zoom / ZOOM_MULTIPLIER
            } else {
                p.zoom
            };
            p.update_zoom(zoom, mouse_position);
            self.schedule_hq_redraw();
        }
        Propagation::Stop
    }

    pub fn mouse_position(&self) -> (f64, f64) {
        self.data.borrow().mouse_position
    }
}

impl ObjectImpl for ImageViewImp {
    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![Signal::builder(SIGNAL_VIEW_RESIZED)
                .param_types([i32::static_type(), i32::static_type()])
                .build()]
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
            move |_, x, y| this.motion_notify_event((x, y))
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
            move |_, context, _, _| this.draw(context)
        ));
    }
}

impl DrawingAreaImpl for ImageViewImp {
    fn resize(&self, _width: i32, _height: i32) {
        // println!("resize {_width} {_height}");
        self.cancel_resize_notify();
        let mut p = self.data.borrow_mut();
        p.apply_zoom();
        self.schedule_resize_notify();
    }
}
