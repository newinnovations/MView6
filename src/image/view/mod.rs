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

pub mod data;
mod imp;
pub mod zoom;

use std::time::SystemTime;

use cairo::{Filter, ImageSurface};
use gdk_pixbuf::Pixbuf;
use gio::Menu;
use glib::{object::Cast, subclass::types::ObjectSubclassIsExt};
use gtk4::{
    gdk::{
        prelude::{DisplayExt, SeatExt, SurfaceExt},
        Display, Rectangle, BUTTON_SECONDARY,
    },
    glib,
    prelude::{GestureSingleExt, NativeExt, PopoverExt, WidgetExt},
    ApplicationWindow, GestureClick, PopoverMenu,
};
use mupdf::Rect;

use crate::backends::thumbnail::model::Annotations;

use super::Image;
pub use data::QUALITY_HIGH;
pub use imp::{SIGNAL_CANVAS_RESIZED, SIGNAL_HQ_REDRAW};
pub use zoom::{ImageZoom, ZoomMode};

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk4::DrawingArea, gtk4::Widget, @implements gtk4::Buildable;
}

impl Default for ImageView {
    fn default() -> Self {
        Self::new()
    }
}

pub enum ViewCursor {
    Normal,
    Hidden,
    Drag,
}

impl ImageView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_image(&self, image: Image) {
        self.set_image_pre(image);
        self.set_image_post(Default::default());
    }

    pub fn set_image_pre(&self, image: Image) {
        let mut p = self.imp().data.borrow_mut();
        self.imp().cancel_animation();
        p.image = image;
        p.zoom.set_rotation(0);
        p.annotations = None;
        p.hover = None;
    }

    pub fn set_image_post(&self, annotations: Option<Annotations>) {
        // dbg!(&annotations);
        let mut p = self.imp().data.borrow_mut();
        p.annotations = annotations;
        p.create_surface();
        self.imp().schedule_animation(&p.image, SystemTime::now());
        p.apply_zoom();
        drop(p);
        self.imp().hq_redraw(false);
        // p.redraw(QUALITY_HIGH);
    }

    pub fn image_modified(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.create_surface();
        p.apply_zoom();
        p.redraw(QUALITY_HIGH); // hq_redraw not needed, because image_modified only used with thumbnail sheets
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        let p = self.imp().data.borrow();
        p.zoom_mode
    }

    pub fn set_zoom_mode(&self, mode: ZoomMode) {
        let mut p = self.imp().data.borrow_mut();
        p.zoom_mode = mode;
    }

    pub fn zoom(&self) -> ImageZoom {
        let p = self.imp().data.borrow();
        p.zoom.clone()
    }

    pub fn clip(&self) -> Rect {
        let p = self.imp().data.borrow();
        let a = self.allocation();
        if let Ok(matrix) = p.zoom.transform_matrix().try_invert() {
            let (x1, y1) = matrix.transform_point(0.0, 0.0);
            let (x2, y2) = matrix.transform_point(a.width() as f64, a.height() as f64);
            Rect {
                x0: x1.min(x2) as f32,
                y0: y1.min(y2) as f32,
                x1: x1.max(x2) as f32,
                y1: y1.max(y2) as f32,
            }
        } else {
            Rect {
                ..Default::default()
            }
        }
    }

    pub fn set_zoomed_surface(&self, surface: ImageSurface) {
        let mut p = self.imp().data.borrow_mut();
        p.redraw(QUALITY_HIGH); // FIXME: handle the removal of existing surfaces better
        p.zoom_surface = Some(surface);
    }

    pub fn set_view_cursor(&self, view_cursor: ViewCursor) {
        match view_cursor {
            ViewCursor::Normal => self.set_cursor_from_name(Some("default")),
            ViewCursor::Hidden => self.set_cursor_from_name(Some("none")),
            ViewCursor::Drag => self.set_cursor_from_name(Some("move")),
        };
    }

    // Operations on image

    pub fn image_id(&self) -> u32 {
        self.imp().data.borrow().image.id()
    }

    pub fn image_size(&self) -> (f64, f64) {
        self.imp().data.borrow().image.size()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        let p = self.imp().data.borrow();
        p.image.draw_pixbuf(pixbuf, dest_x, dest_y);
    }

    pub fn rotate(&self, angle: i32) {
        let mut p = self.imp().data.borrow_mut();
        p.zoom.add_rotation(angle);
        p.apply_zoom();
        p.redraw(QUALITY_HIGH);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.imp().data.borrow().image.has_tag(tag)
    }

    pub fn add_context_menu(&self, menu: Menu) {
        let gesture = GestureClick::new();
        gesture.set_button(BUTTON_SECONDARY); // Right mouse button

        let popup_menu = PopoverMenu::from_model(Some(&menu));
        popup_menu.set_parent(self); // Set the window as the parent
        popup_menu.set_can_focus(false);

        let window_weak = glib::clone::Downgrade::downgrade(&self);
        gesture.connect_released(move |_gesture, _n_press, _x, _y| {
            if let Some(window) = window_weak.upgrade() {
                if let Some(popup) = window
                    .first_child()
                    .and_then(|child| child.downcast::<PopoverMenu>().ok())
                {
                    // Position the menu at the right-click location
                    let (x, y) = window.imp().mouse_position();
                    // println!("current {} {}", x, y);
                    // match window.get_window_relative_cursor_position() {
                    //     Ok(position) => {
                    //         println!("determined {} {}", position.0, position.1)
                    //     }
                    //     Err(err) => println!("error {}", err),
                    // }
                    let rect = Rectangle::new(x as i32, y as i32, 1, 1);
                    popup.set_pointing_to(Some(&rect));
                    popup.popup();
                }
            }
        });

        self.add_controller(gesture);
    }

    #[allow(dead_code)]
    fn get_window_relative_cursor_position(&self) -> Result<(f64, f64), &'static str> {
        let display = Display::default().ok_or("Display::default")?;
        let seat = display.default_seat().ok_or("display.default_seat")?;
        let native = self.native().ok_or("self.native")?;
        let surface = native.surface().ok_or("native.surface")?;
        let device = seat.pointer().ok_or("seat.pointer")?;
        let position = surface
            .device_position(&device)
            .ok_or("surface.device_position")?;
        let root = self.root().ok_or("self.root")?;
        let root = root
            .downcast::<ApplicationWindow>()
            .map_err(|_| "root.downcast")?;
        let widget_bounds = self.compute_bounds(&root).ok_or("compute_bounds")?;
        let relative_x = position.0 - widget_bounds.x() as f64 - 14.0;
        let relative_y = position.1 - widget_bounds.y() as f64 - 12.0;
        let relative_x = relative_x.clamp(0.0, widget_bounds.width() as f64);
        let relative_y = relative_y.clamp(0.0, widget_bounds.height() as f64);
        Ok((relative_x, relative_y))
    }

    // redraw stuff
    pub fn apply_zoom(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.apply_zoom();
    }

    pub fn redraw(&self, quality: Filter) {
        // BorrowMutError: to reproduce press 'q' while hovering image in thumbnail sheet
        match self.imp().data.try_borrow_mut() {
            Ok(mut p) => p.redraw(quality),
            Err(e) => eprintln!("Failed to redraw: {e}"),
        }
    }
}
