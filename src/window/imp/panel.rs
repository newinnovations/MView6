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
use gtk4::{
    glib, prelude::*, Align, Box, Button, Justification, Label, MenuButton, Orientation, Overlay,
    Revealer,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::file_view::Direction;
use crate::image::view::ImageView;
use crate::rect::PointD;
use crate::util::remove_source_id;
use crate::window::imp::MViewWindowImp;

static PANEL_TIMEOUT: u32 = 5; // seconds
static PANEL_TRANSITION: u32 = 200; // milliseconds
static PANEL_DRAG_THRESHOLD: f64 = 2.0; // pixels

macro_rules! connect_panel_button {
    ($button:expr, $window:expr, $timer:expr, $window_var:ident => $action:expr) => {{
        let win_weak = $window.downgrade();
        let timer_strong = $timer.clone();
        $button.connect_clicked(move |_| {
            if let Some($window_var) = win_weak.upgrade() {
                reset_timer(&timer_strong);
                $action;
            }
        });
    }};
}

#[derive(Debug)]
pub struct Panel {
    pub overlay: Overlay,
    forward_button: Button,
    can_enter: Cell<bool>,
    slideshow_mode: Cell<bool>,
    normal_buttons: Vec<Button>,
    slideshow_buttons: Vec<Button>,
}

impl Panel {
    pub fn enable_enter(&self, enabled: bool) {
        if self.can_enter.get() != enabled {
            self.can_enter.set(enabled);
            self.set_visibility();
        }
    }

    pub fn enable_slideshow_mode(&self, enabled: bool) {
        if self.slideshow_mode.get() != enabled {
            self.slideshow_mode.set(enabled);
            self.set_visibility();
        }
    }

    fn set_visibility(&self) {
        let can_enter = self.can_enter.get();
        let slideshow_mode = self.slideshow_mode.get();

        for button in &self.normal_buttons {
            button.set_visible(!slideshow_mode);
        }

        for button in &self.slideshow_buttons {
            button.set_visible(slideshow_mode);
        }

        self.forward_button
            .set_visible(can_enter && !slideshow_mode);
    }

    pub fn create(mview_window: &MViewWindowImp, image_view: &ImageView, menu: &Menu) -> Self {
        // Create button panel
        let panel_box = Box::new(Orientation::Vertical, 5);
        panel_box.add_css_class("panel");
        let row_1 = Box::new(Orientation::Horizontal, 5);
        let row_2 = Box::new(Orientation::Horizontal, 5);
        panel_box.append(&row_1);
        panel_box.append(&row_2);

        // Create buttons with icons
        let fullscreen_button = create_icon_button("panel-fullscreen", "Toggle fullscreen");
        let previous_button = create_icon_button("panel-up", "Previous in list");
        let next_button = create_icon_button("panel-down", "Next in list");
        let back_button = create_icon_button("panel-previous", "Go to parent");
        let forward_button = create_icon_button("panel-next", "Open directory/archive");
        let filelist_button = create_icon_button("panel-dual", "Toggle file list");
        let start_button = create_icon_button("panel-start", "Start slideslow");
        let stop_button = create_icon_button("panel-stop", "Stop slideslow");
        // let zoom_mode_button = create_text_button("Zoom\nmode", "Change zoom mode");
        // let zoom_in_button = create_text_button("Zoom\n<span size=\"large\">+</span>", "Zoom in");
        // let zoom_out_button = create_text_button("Zoom\n<span size=\"large\">-</span>", "Zoom out");
        let zoom_mode_button = create_icon_button("panel-zoom-fit", "Change zoom mode");
        let zoom_in_button = create_icon_button("panel-zoom-in", "Zoom in");
        let zoom_out_button = create_icon_button("panel-zoom-out", "Zoom out");
        let filter_button = create_icon_button("panel-filter", "Navigation filter");
        let int_1_button = create_text_button("<big>1</big>\nsec", "Set interval to 1 second");
        let int_3_button = create_text_button("<big>3</big>\nsec", "Set interval to 3 seconds");
        let int_5_button = create_text_button("<big>5</big>\nsec", "Set interval to 5 seconds");
        let int_10_button = create_text_button("<big>10</big>\nsec", "Set interval to 10 seconds");
        let int_30_button = create_text_button("<big>30</big>\nsec", "Set interval to 30 seconds");
        let int_60_button = create_text_button("<big>60</big>\nsec", "Set interval to 1 minute");
        let menu_button = MenuButton::builder()
            .icon_name("panel-menu") // hamburger icon
            .can_focus(false)
            .css_classes(["panel_button"])
            .build();
        menu_button.set_menu_model(Some(menu));

        // Add buttons to panel
        row_1.append(&fullscreen_button);
        row_1.append(&menu_button);
        row_1.append(&zoom_mode_button);
        row_1.append(&zoom_in_button);
        row_1.append(&back_button);
        row_1.append(&forward_button);
        row_1.append(&int_1_button);
        row_1.append(&int_5_button);
        row_1.append(&int_30_button);

        row_2.append(&filelist_button);
        row_2.append(&filter_button);
        row_2.append(&start_button);
        row_2.append(&stop_button);
        row_2.append(&zoom_out_button);
        row_2.append(&previous_button);
        row_2.append(&next_button);
        row_2.append(&int_3_button);
        row_2.append(&int_10_button);
        row_2.append(&int_60_button);

        // Create revealer to show/hide panel with animation
        let revealer = Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);
        revealer.set_transition_duration(PANEL_TRANSITION);
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
                    panel_box.set_orientation(Orientation::Vertical);
                    row_1.set_orientation(Orientation::Horizontal);
                    row_2.set_orientation(Orientation::Horizontal);
                } else {
                    // Tall or square screen - vertical layout
                    panel_box.set_orientation(Orientation::Horizontal);
                    row_1.set_orientation(Orientation::Vertical);
                    row_2.set_orientation(Orientation::Vertical);
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
            image_view,
            #[strong]
            revealer,
            #[strong]
            hide_timer,
            #[strong]
            panel_visible,
            move |_, _, x, y| {
                let drag = mouse_on_click.borrow().distance(&PointD::new(x, y));
                if drag < PANEL_DRAG_THRESHOLD {
                    let mut visible = panel_visible.borrow_mut();
                    *visible = if image_view.measure_active() | (x > 150.0) | (y > 150.0) {
                        false
                    } else {
                        !*visible
                    };
                    revealer.set_reveal_child(*visible);

                    // Cancel existing timer if any
                    reset_timer(&hide_timer);

                    // Start new timer if panel is now visible
                    if *visible {
                        let revealer_timer = revealer.clone();
                        let panel_visible_timer = panel_visible.clone();
                        let timer_id = glib::timeout_add_seconds_local(
                            PANEL_TIMEOUT,
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
        if let Some(popover) = menu_button.popover() {
            popover.connect_show(clone!(
                #[strong]
                hide_timer,
                move |_| {
                    reset_timer(&hide_timer);
                }
            ));
        }

        connect_panel_button!(previous_button, mview_window, hide_timer, w => {
            w.navigate_item_filter(Direction::Up, 1);
        });

        connect_panel_button!(next_button, mview_window, hide_timer, w => {
            w.navigate_item_filter(Direction::Down, 1);
        });

        connect_panel_button!(fullscreen_button, mview_window, hide_timer, w => {
            w.toggle_fullscreen();
        });

        connect_panel_button!(back_button, mview_window, hide_timer, w => {
            w.dir_leave();
        });

        connect_panel_button!(forward_button, mview_window, hide_timer, w => {
            w.dir_enter();
        });

        connect_panel_button!(filelist_button, mview_window, hide_timer, w => {
            w.toggle_pane_files();
        });

        connect_panel_button!(zoom_mode_button, mview_window, hide_timer, w => {
            w.toggle_zoom();
        });

        connect_panel_button!(zoom_in_button, mview_window, hide_timer, w => {
            w.zoom_in();
        });

        connect_panel_button!(zoom_out_button, mview_window, hide_timer, w => {
            w.zoom_out();
        });

        connect_panel_button!(filter_button, mview_window, hide_timer, w => {
            w.filter_dialog();
        });

        connect_panel_button!(start_button, mview_window, hide_timer, w => {
            w.set_slideshow_active(true);
        });

        connect_panel_button!(stop_button, mview_window, hide_timer, w => {
            w.set_slideshow_active(false);
        });

        connect_panel_button!(int_1_button, mview_window, hide_timer, w => {
            w.set_slideshow_interval(1);
        });

        connect_panel_button!(int_3_button, mview_window, hide_timer, w => {
            w.set_slideshow_interval(3);
        });

        connect_panel_button!(int_5_button, mview_window, hide_timer, w => {
            w.set_slideshow_interval(5);
        });

        connect_panel_button!(int_10_button, mview_window, hide_timer, w => {
            w.set_slideshow_interval(10);
        });

        connect_panel_button!(int_30_button, mview_window, hide_timer, w => {
            w.set_slideshow_interval(30);
        });

        connect_panel_button!(int_60_button, mview_window, hide_timer, w => {
            w.set_slideshow_interval(60);
        });

        let panel = Self {
            overlay,
            forward_button,
            can_enter: true.into(),
            slideshow_mode: false.into(),
            // common_buttons: vec![
            //     fullscreen_button,
            //     zoom_mode_button,
            //     filelist_button,
            //     filter_button,
            // ],
            normal_buttons: vec![
                start_button,
                next_button,
                previous_button,
                back_button,
                zoom_in_button,
                zoom_out_button,
            ],
            slideshow_buttons: vec![
                stop_button,
                int_1_button,
                int_3_button,
                int_5_button,
                int_10_button,
                int_30_button,
                int_60_button,
            ],
        };

        panel.set_visibility();

        panel
    }
}

fn create_icon_button(icon_name: &str, tooltip: &str) -> Button {
    let button = Button::from_icon_name(icon_name);
    button.set_tooltip_text(Some(tooltip));
    button.add_css_class("panel_button");
    button
}

fn create_text_button(markup: &str, tooltip: &str) -> Button {
    let button = Button::new();
    button.set_tooltip_text(Some(tooltip));
    button.add_css_class("panel_button");
    button.add_css_class("panel_text_button");

    // Create a label with markup
    let label = Label::new(None);
    label.set_markup(markup);
    label.set_justify(Justification::Center);
    label.set_halign(Align::Center);

    // Set the label as the button's child
    button.set_child(Some(&label));

    button
}

fn reset_timer(hide_timer: &Rc<RefCell<Option<glib::SourceId>>>) {
    if let Some(timer_id) = hide_timer.borrow_mut().take() {
        if remove_source_id(&timer_id).is_err() {
            eprintln!("reset_timer failed");
        }
    }
}
