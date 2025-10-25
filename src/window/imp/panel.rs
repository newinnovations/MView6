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

use gio::Menu;
use glib::clone;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::MenuButton;
use gtk4::{Box, Button, Orientation, Overlay, Revealer};
use std::cell::RefCell;
use std::rc::Rc;

use crate::file_view::Direction;
use crate::file_view::Filter;
use crate::image::view::ImageView;
use crate::rect::PointD;
use crate::window::imp::MViewWindowImp;

pub fn create_overlay_button_panel(
    mview_window: &MViewWindowImp,
    image_view: &ImageView,
    menu: &Menu,
) -> (Overlay, Button) {
    // Create button panel
    let panel_box = Box::new(Orientation::Horizontal, 5);
    panel_box.add_css_class("panel");

    // Create buttons with icons
    let fullscreen_button = create_button("view-fullscreen-symbolic", "Toggle fullscreen");
    let previous_button = create_button("go-up-symbolic", "Previous in list");
    let next_button = create_button("go-down-symbolic", "Next in list");
    let back_button = create_button("go-previous-symbolic", "Go to parent");
    let forward_button = create_button("go-next-symbolic", "Open directory/archive");
    let filelist_button = create_button("view-dual-symbolic", "Toggle file list");
    let menu_button = MenuButton::builder()
        .icon_name("open-menu-symbolic") // hamburger icon
        .can_focus(false)
        .build();
    menu_button.set_menu_model(Some(menu));

    // Add buttons to panel
    panel_box.append(&fullscreen_button);
    panel_box.append(&previous_button);
    panel_box.append(&next_button);
    panel_box.append(&filelist_button);
    panel_box.append(&back_button);
    panel_box.append(&menu_button);
    panel_box.append(&forward_button);

    // Create revealer to show/hide panel with animation
    let revealer = Revealer::new();
    revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
    revealer.set_transition_duration(200);
    revealer.set_child(Some(&panel_box));
    revealer.set_reveal_child(false);
    revealer.set_halign(gtk4::Align::Start);
    revealer.set_valign(gtk4::Align::Start);

    // Create overlay to place panel over drawing area
    let overlay = Overlay::new();
    overlay.set_child(Some(image_view));
    overlay.add_overlay(&revealer);

    // Track panel visibility
    let panel_visible = Rc::new(RefCell::new(false));

    // Track the auto-hide timer
    let hide_timer = Rc::new(RefCell::new(None::<glib::SourceId>));

    // Track dragging
    let mouse_on_click = Rc::new(RefCell::new(PointD::default()));

    // Handle drawing area size changes to adjust panel orientation
    image_view.connect_resize(clone!(
        #[strong]
        panel_box,
        move |_, width, height| {
            let aspect_ratio = width as f64 / height as f64;

            // Switch between horizontal and vertical layout based on aspect ratio
            if aspect_ratio > 1.5 {
                // Wide screen - horizontal layout
                panel_box.set_orientation(Orientation::Horizontal);
            } else {
                // Tall or square screen - vertical layout
                panel_box.set_orientation(Orientation::Vertical);
            }
        }
    ));

    // Add click gesture to drawing area
    let gesture = gtk4::GestureClick::new();

    gesture.connect_pressed(clone!(
        #[strong]
        mouse_on_click,
        move |_, _, x, y| {
            mouse_on_click.replace(PointD::new(x, y));
        }
    ));

    gesture.connect_released(clone!(
        #[strong]
        revealer,
        #[strong]
        hide_timer,
        #[strong]
        panel_visible,
        move |_, _, x, y| {
            let drag = mouse_on_click.borrow().distance(&PointD::new(x, y));
            if drag < 1.0 {
                let mut visible = panel_visible.borrow_mut();
                *visible = !*visible;
                revealer.set_reveal_child(*visible);

                // Cancel existing timer if any
                reset_timer(&hide_timer);

                // Start new timer if panel is now visible
                if *visible {
                    let revealer_timer = revealer.clone();
                    let panel_visible_timer = panel_visible.clone();
                    let timer_id = glib::timeout_add_seconds_local(
                        2,
                        clone!(
                            #[strong]
                            hide_timer,
                            move || {
                                revealer_timer.set_reveal_child(false);
                                *panel_visible_timer.borrow_mut() = false;
                                *hide_timer.borrow_mut() = None; // Clear the timer reference
                                glib::ControlFlow::Break
                            }
                        ),
                    );
                    *hide_timer.borrow_mut() = Some(timer_id);
                }
            }
        }
    ));

    image_view.add_controller(gesture);

    // Button actions
    fullscreen_button.connect_clicked(clone!(
        #[weak]
        mview_window,
        #[strong]
        hide_timer,
        move |_| {
            reset_timer(&hide_timer);
            mview_window.toggle_fullscreen();
        }
    ));

    if let Some(popover) = menu_button.popover() {
        popover.connect_show(clone!(
            #[strong]
            hide_timer,
            move |_| {
                reset_timer(&hide_timer);
            }
        ));
    }

    back_button.connect_clicked(clone!(
        #[weak]
        mview_window,
        #[strong]
        hide_timer,
        move |_| {
            reset_timer(&hide_timer);
            mview_window.dir_leave();
        }
    ));

    forward_button.connect_clicked(clone!(
        #[weak]
        mview_window,
        #[strong]
        hide_timer,
        move |_| {
            reset_timer(&hide_timer);
            mview_window.dir_enter();
        }
    ));

    previous_button.connect_clicked(clone!(
        #[weak]
        mview_window,
        #[strong]
        hide_timer,
        move |_| {
            reset_timer(&hide_timer);
            mview_window
                .widgets()
                .file_view
                .navigate_item(Direction::Up, Filter::None, 1);
        }
    ));

    next_button.connect_clicked(clone!(
        #[weak]
        mview_window,
        #[strong]
        hide_timer,
        move |_| {
            reset_timer(&hide_timer);
            mview_window
                .widgets()
                .file_view
                .navigate_item(Direction::Down, Filter::None, 1);
        }
    ));

    filelist_button.connect_clicked(clone!(
        #[weak]
        mview_window,
        #[strong]
        hide_timer,
        move |_| {
            reset_timer(&hide_timer);
            mview_window.toggle_pane_files();
        }
    ));

    (overlay, forward_button)
}

fn create_button(icon_name: &str, tooltip: &str) -> Button {
    let button = Button::from_icon_name(icon_name);
    button.set_tooltip_text(Some(tooltip));
    button
}

fn reset_timer(hide_timer: &Rc<RefCell<Option<glib::SourceId>>>) {
    if let Some(timer_id) = hide_timer.borrow_mut().take() {
        timer_id.remove();
    }
}
