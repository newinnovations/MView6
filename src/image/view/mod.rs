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

use std::time::SystemTime;

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

use crate::backends::thumbnail::model::Annotations;

use super::Image;
pub use imp::SIGNAL_VIEW_RESIZED;

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk4::DrawingArea, gtk4::Widget, @implements gtk4::Buildable;
}

impl Default for ImageView {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy)]
pub enum ZoomMode {
    #[default]
    NotSpecified,
    NoZoom,
    Fit,
    Fill,
    Max,
}

impl From<&str> for ZoomMode {
    fn from(value: &str) -> Self {
        match value {
            "nozoom" => ZoomMode::NoZoom,
            "fit" => ZoomMode::Fit,
            "fill" => ZoomMode::Fill,
            "max" => ZoomMode::Max,
            _ => ZoomMode::NotSpecified,
        }
    }
}

impl From<ZoomMode> for &str {
    fn from(value: ZoomMode) -> Self {
        match value {
            ZoomMode::NotSpecified => "",
            ZoomMode::NoZoom => "nozoom",
            ZoomMode::Fit => "fit",
            ZoomMode::Fill => "fill",
            ZoomMode::Max => "max",
        }
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
        // p.image.crop_to_max_size();
        p.rotation = 0;
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
    }

    pub fn image_modified(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.create_surface();
        // p.redraw(QUALITY_HIGH);
        p.apply_zoom();
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        let p = self.imp().data.borrow();
        p.zoom_mode
    }

    pub fn set_zoom_mode(&self, mode: ZoomMode) {
        let mut p = self.imp().data.borrow_mut();
        p.zoom_mode = mode;
        p.apply_zoom();
    }

    pub fn offset(&self) -> (f64, f64) {
        let p = self.imp().data.borrow();
        (p.xofs, p.yofs)
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

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        let p = self.imp().data.borrow();
        p.image.draw_pixbuf(pixbuf, dest_x, dest_y);
    }

    pub fn rotate(&self, angle: i32) {
        let mut p = self.imp().data.borrow_mut();
        p.rotation = (p.rotation + angle).rem_euclid(360);
        p.image.rotate(angle);
        p.create_surface();
        p.apply_zoom();
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
}
