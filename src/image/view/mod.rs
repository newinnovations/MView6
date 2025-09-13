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

use crate::{
    backends::thumbnail::model::Annotations,
    content::{Content, ContentData},
    file_view::Direction,
    image::{provider::surface::SurfaceData, view::data::TransparencyMode},
    rect::{RectD, SizeD},
    window::imp::MViewWidgets,
};

pub use data::redraw::RedrawReason;
pub use data::zoom::{Zoom, ZoomMode};
pub use data::QUALITY_HIGH;

pub const SIGNAL_CANVAS_RESIZED: &str = "event-canvas-resized";
pub const SIGNAL_NAVIGATE: &str = "event-navigate";

glib::wrapper! {
    pub struct ImageView(ObjectSubclass<imp::ImageViewImp>)
        @extends gtk4::DrawingArea, gtk4::Widget, @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
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
        let view: Self = glib::Object::builder().build();
        let mut p = view.imp().data.borrow_mut();
        p.zoom_mode = ZoomMode::Fill;
        drop(p);
        view
    }

    pub fn init(&self, widgets: &MViewWidgets) {
        let mut p = self.imp().data.borrow_mut();
        p.rb_sender = Some(widgets.rt_sender.clone());
    }

    pub fn set_image(&self, image: Content) {
        self.set_image_pre(image);
        self.set_image_post(Default::default());
    }

    pub fn set_image_pre(&self, image: Content) {
        let mut p = self.imp().data.borrow_mut();
        self.imp().cancel_animation();
        p.content = image;
        p.zoom.set_rotation(0);
        p.zoom_overlay = None;
        p.annotations = None;
        p.hover = None;
    }

    pub fn set_image_post(&self, annotations: Option<Annotations>) {
        // dbg!(&annotations);
        let mut p = self.imp().data.borrow_mut();
        p.annotations = annotations;
        self.imp().schedule_animation(&p.content, SystemTime::now());
        p.apply_zoom();
        p.redraw(RedrawReason::ContentPost);
    }

    pub fn image_modified(&self) {
        let mut p = self.imp().data.borrow_mut();
        p.apply_zoom();
        p.redraw(RedrawReason::ContentChanged);
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        let p = self.imp().data.borrow();
        p.zoom_mode
    }

    pub fn set_zoom_mode(&self, mode: ZoomMode) {
        let mut p = self.imp().data.borrow_mut();
        p.zoom_mode = mode;
        p.apply_zoom();
        p.redraw(RedrawReason::ZoomSettingChanged);
    }

    pub fn set_transparency_mode(&self, mode: TransparencyMode) {
        let mut p = self.imp().data.borrow_mut();
        p.transparency_mode = mode;
        p.redraw(RedrawReason::TransparencyBackgroundChanged);
    }

    pub fn zoom(&self) -> Zoom {
        let p = self.imp().data.borrow();
        p.zoom.clone()
    }

    pub fn hq_render_reply(
        &self,
        image_id: u32,
        surface_data: SurfaceData,
        zoom: Zoom,
        viewport: RectD,
    ) {
        let mut p = self.imp().data.borrow_mut();
        p.hq_render_reply(image_id, surface_data, zoom, viewport);
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
        self.imp().data.borrow().content.id()
    }

    pub fn image_size(&self) -> SizeD {
        self.imp().data.borrow().content.size()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        let p = self.imp().data.borrow();
        p.content.draw_pixbuf(pixbuf, dest_x, dest_y);
    }

    pub fn rotate(&self, angle: i32) {
        let mut p = self.imp().data.borrow_mut();
        p.zoom.add_rotation(angle);
        p.apply_zoom();
        p.zoom_overlay = None;
        p.redraw(RedrawReason::RotationChanged);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.imp().data.borrow().content.has_tag(tag)
    }

    pub fn navigate_page(&self, direction: Direction, count: u32) -> bool {
        let mut p = self.imp().data.borrow_mut();
        if let ContentData::Paginated(paginated) = &mut p.content.data {
            let page_changed = paginated.navigate_page(direction, count as usize);
            if page_changed {
                p.redraw(RedrawReason::PageChanged);
            }
            page_changed
        } else {
            false
        }
    }

    pub fn on_sort_changed(&self, new_sort: &str) {
        dbg!(new_sort);
        let mut p = self.imp().data.borrow_mut();
        if p.content.sort(new_sort) {
            p.redraw(RedrawReason::SortChanged);
        }
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
                    let pos = window.imp().mouse_position();
                    // println!("current {} {}", x, y);
                    // match window.get_window_relative_cursor_position() {
                    //     Ok(position) => {
                    //         println!("determined {} {}", position.0, position.1)
                    //     }
                    //     Err(err) => println!("error {}", err),
                    // }
                    let rect = Rectangle::new(pos.x() as i32, pos.y() as i32, 1, 1);
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
