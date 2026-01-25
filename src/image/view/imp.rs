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

use super::{data::ImageViewData, ImageView, ViewCursor};
use crate::{
    classification::Preference,
    content::Content,
    image::{
        colors::{CairoColorExt, Color},
        draw::transparency_background,
        view::{
            data::{
                zoom::{ZOOM_MULTIPLIER, ZOOM_MULTIPLIER_FAST},
                TransparencyMode,
            },
            measure::{MeasureTool, MeasurementState},
            RedrawReason, SIGNAL_CANVAS_RESIZED, SIGNAL_NAVIGATE, SIGNAL_SHOWN,
        },
    },
    rect::{PointD, RectD, SizeI},
    util::remove_source_id,
};
use cairo::{Context, Extend, FillRule, SurfacePattern};
use gio::prelude::StaticType;
use glib::{clone, object::ObjectExt, subclass::Signal, ControlFlow, Propagation, SourceId};
use gtk4::{
    gdk::ModifierType,
    prelude::{DrawingAreaExtManual, EventControllerExt, GestureSingleExt, WidgetExt},
    subclass::prelude::*,
    EventControllerMotion, EventControllerScroll, EventControllerScrollFlags,
};

#[derive(Default)]
pub struct ImageViewImp {
    pub(super) data: RefCell<ImageViewData>,
    animation_timeout_id: RefCell<Option<SourceId>>,
    pub(super) window_size: Cell<SizeI>,
    pub(super) measure_tool: MeasureTool,
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
            if let Err(e) = remove_source_id(&id) {
                println!("remove_source_id: {e}");
            }
        }
    }

    pub fn schedule_animation(&self, content: &Content, ts_previous_cb: SystemTime) {
        if let Some(animation) = content.animation() {
            if let Some(interval) = animation.delay_time(ts_previous_cb) {
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
        if let Some(animation) = p.content.animation_mut() {
            if animation.advance(SystemTime::now()) {
                self.schedule_animation(&p.content, start);
                p.redraw(RedrawReason::AnimationCallback);
            }
        }
    }

    fn draw(&self, context: &Context) {
        let p = self.data.borrow();
        let z = &p.zoom;

        let image = p.image();

        let _ = context.save();

        context.set_fill_rule(FillRule::EvenOdd);

        let viewport = clip_extents_to_rect(context);
        let intersect = z.intersection_screen_coord(&viewport);
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

        // NOTE: uses image.transparency_mode to see if it needs to override user setting
        if image.has_alpha() {
            let transparency_mode = if p.content.transparency_mode == TransparencyMode::NotSpecified
            {
                p.transparency_mode
            } else {
                p.content.transparency_mode
            };

            match transparency_mode {
                TransparencyMode::White => context.color(Color::White),
                TransparencyMode::Black => context.color(Color::Black),
                _ => {
                    if let Some(checkerboard) = &p.checkerboard {
                        // Create a checkerboard pattern
                        let pattern = SurfacePattern::create(checkerboard);
                        pattern.set_extend(Extend::Repeat);
                        let _ = context.set_source(&pattern);
                    } else {
                        context.color(Color::Black);
                    }
                }
            }
            // make the transparency background one pixel smaller at every side to not extend the
            // images in case of rounding errors
            context.rectangle(
                intersect.x0 + 1.0,
                intersect.y0 + 1.0,
                intersect.width() - 2.0,
                intersect.height() - 2.0,
            );
            let _ = context.fill();
        }

        // Viewport offset is handled in the transformation matrix so drawing here happens
        // at the virtual origin (0.0, 0.0)
        context.transform(image.transform_matrix(&p.zoom));
        image.draw(context, p.quality);
        self.draw_annotations(context);

        if self.measure_tool.state() != MeasurementState::Idle {
            let _ = context.restore();
            self.measure_tool.draw(context, z, &self.mouse_position());
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
                match annotation.entry.preference() {
                    Preference::Liked => context.set_source_rgb(0.0, 1.0, 0.0),
                    Preference::Disliked => context.set_source_rgb(1.0, 1.0, 0.0),
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

    fn button_press_event(&self, position: PointD, n_press: i32) {
        let mut p = self.data.borrow_mut();
        if n_press == 1 {
            if self.measure_tool.is_tracking() {
                self.measure_tool
                    .set_point(p.zoom.screen_to_image(&position));
                p.redraw(RedrawReason::Measurement);
            } else if p.drag.is_none() && p.content.is_movable() {
                p.drag = Some(position - p.zoom.origin());
                self.obj().set_view_cursor(ViewCursor::Drag);
            }
        } else if n_press == 2 {
            let image_postion = p.zoom.screen_to_image(&position);
            let reference = p.content.double_click(image_postion);
            if !reference.backend.is_none() {
                self.obj().emit_by_name::<()>(
                    SIGNAL_NAVIGATE,
                    &[
                        &reference.backend.name(),
                        &reference.backend.path(),
                        &reference.item.to_string_repr(),
                    ],
                );
            }
        }
    }

    fn button_release_event(&self) {
        let mut p = self.data.borrow_mut();
        if p.drag.is_some() {
            p.drag = None;
            self.obj().set_view_cursor(ViewCursor::Normal);
        }
    }

    fn motion_notify_event(&self, position: PointD) {
        let mut p = self.data.borrow_mut();
        p.mouse_position = position;
        if self.measure_tool.is_tracking() {
            p.redraw(RedrawReason::Measurement);
        } else if let Some(annotations) = &p.annotations {
            let index = annotations.index_at(position - p.zoom.origin());
            if index != p.hover {
                p.hover = index;
                p.redraw(RedrawReason::AnnotationChanged);
            }
        } else if let Some(drag) = p.drag {
            p.zoom.set_origin(position - drag);
            p.redraw(RedrawReason::InteractiveDrag);
        }
    }

    fn motion_leave_event(&self) {
        let mut p = self.data.borrow_mut();
        if p.hover.is_some() {
            p.hover = None;
            p.redraw(RedrawReason::AnnotationChanged);
        }
    }

    fn scroll_event(&self, dy: f64, modifier: ModifierType) -> Propagation {
        let mut p = self.data.borrow_mut();
        let mouse_position = p.mouse_position;
        let multiplier = if modifier.contains(ModifierType::CONTROL_MASK) {
            ZOOM_MULTIPLIER_FAST
        } else {
            ZOOM_MULTIPLIER
        };
        if p.content.is_movable() {
            let zoom = if dy < -0.01 {
                p.zoom.scale() * multiplier
            } else if dy > 0.01 {
                p.zoom.scale() / multiplier
            } else {
                p.zoom.scale()
            };
            p.update_zoom(zoom, mouse_position);
            p.redraw(RedrawReason::InteractiveZoom);
        }
        Propagation::Stop
    }

    pub fn mouse_position(&self) -> PointD {
        self.data.borrow().mouse_position
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
                Signal::builder(SIGNAL_NAVIGATE)
                    .param_types([
                        String::static_type(),
                        String::static_type(),
                        String::static_type(),
                    ])
                    .build(),
                Signal::builder(SIGNAL_SHOWN).build(),
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
            move |_, x, y| this.motion_notify_event(PointD::new(x, y))
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
            move |controller, _dx, dy| {
                let modifiers = controller.current_event_state();
                this.scroll_event(dy, modifiers)
            }
        ));

        let gesture_click = gtk4::GestureClick::new();
        gesture_click.set_button(1);
        gesture_click.connect_pressed(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, n_press, x, y| this.button_press_event(PointD::new(x, y), n_press)
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
        p.checkerboard = transparency_background().ok();

        self.obj().set_draw_func(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, context, _width, _height| this.draw(context)
        ));
    }
}

impl DrawingAreaImpl for ImageViewImp {
    fn resize(&self, width: i32, height: i32) {
        let new_size = SizeI::new(width, height);
        let current_size = self.window_size.get();
        if current_size != new_size {
            self.window_size.set(new_size);

            self.obj()
                .emit_by_name::<()>(SIGNAL_CANVAS_RESIZED, &[&width, &height]);

            let mut p = self.data.borrow_mut();
            p.apply_zoom();
            p.redraw(RedrawReason::CanvasResized);
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
