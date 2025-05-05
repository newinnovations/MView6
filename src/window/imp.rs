// MView6 -- Opiniated image browser written in Rust and GTK4
//
// Copyright (c) 2024 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU General Public License as published by the Free Software Foundation, either version 3
// of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

mod backend;
mod keyboard;
mod mouse;
mod navigate;

use crate::{
    backends::{
        document::PageMode,
        thumbnail::{
            processing::{handle_thumbnail_result, start_thumbnail_task},
            Message, TCommand,
        },
        Backend,
    },
    file_view::{FileView, Filter, Selection, Sort},
    image::view::{ImageView, ZoomMode, SIGNAL_VIEW_RESIZED},
    info_view::InfoView,
};
use async_channel::Sender;
use glib::{clone, closure_local};
use gtk4::{
    glib::{self, Propagation},
    prelude::*,
    subclass::prelude::*,
    EventControllerKey, ScrolledWindow,
};
use std::cell::{Cell, OnceCell, RefCell};

#[derive(Debug)]
pub struct MViewWidgets {
    hbox: gtk4::Box,
    file_widget: ScrolledWindow,
    file_view: FileView,
    info_widget: ScrolledWindow,
    info_view: InfoView,
    image_view: ImageView,
    pub sender: Sender<Message>,
}

#[derive(Debug, Default)]
pub struct MViewWindowImp {
    widget_cell: OnceCell<MViewWidgets>,
    backend: RefCell<Box<dyn Backend>>,
    full_screen: Cell<bool>,
    skip_loading: Cell<bool>,
    thumbnail_size: Cell<i32>,
    current_sort: Cell<Sort>,
    page_mode: Cell<PageMode>,
}

#[glib::object_subclass]
impl ObjectSubclass for MViewWindowImp {
    const NAME: &'static str = "MViewWindow";
    type Type = super::MViewWindow;
    type ParentType = gtk4::ApplicationWindow;
}

impl MViewWindowImp {
    fn widgets(&self) -> &MViewWidgets {
        self.widget_cell.get().unwrap()
    }

    pub fn show_files_widget(&self, show: bool) {
        let w = self.widgets();
        if w.file_widget.is_visible() != show {
            w.file_widget.set_visible(show);
            self.update_margins();
        }
    }

    pub fn show_info_widget(&self, show: bool) {
        let w = self.widgets();
        if w.info_widget.is_visible() != show {
            w.info_widget.set_visible(show);
            self.update_margins();
        }
    }

    pub fn update_margins(&self) {
        let w = self.widgets();
        let border = if w.file_widget.is_visible() || w.info_widget.is_visible() {
            8
        } else {
            0
        };
        w.hbox.set_spacing(0);
        w.file_widget.set_margin_start(border);
        w.file_widget.set_margin_top(border);
        w.file_widget.set_margin_bottom(border);
        w.image_view.set_margin_start(border);
        w.image_view.set_margin_top(border);
        w.image_view.set_margin_bottom(border);
        w.image_view.set_margin_end(border);
        w.info_widget.set_margin_end(border);
        w.info_widget.set_margin_top(border);
        w.info_widget.set_margin_bottom(border);
        w.file_view.set_extended(!w.info_widget.is_visible());
    }

    pub fn step_size(&self) -> u32 {
        if self.backend.borrow().is_doc() {
            match self.page_mode.get() {
                PageMode::Single => 1,
                PageMode::DualOdd => 2,
                PageMode::DualEven => 2,
            }
        } else {
            1
        }
    }
}

impl ObjectImpl for MViewWindowImp {
    fn constructed(&self) {
        self.parent_constructed();
        self.thumbnail_size.set(175);
        self.current_sort.set(Sort::sort_on_category());

        let window = self.obj();

        window.set_title(Some("MView6"));
        // window.set_position(gtk4::WindowPosition::Center); TODO
        window.set_default_size(1280, 720);

        let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);

        let file_widget = ScrolledWindow::new();
        // files_widget.set_shadow_type(gtk4::ShadowType::EtchedIn); TODO
        file_widget.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        file_widget.set_can_focus(false);
        hbox.append(&file_widget);

        let file_view = FileView::new();
        file_view.set_vexpand(true);
        file_view.set_fixed_height_mode(true);
        file_view.set_can_focus(false);
        file_widget.set_child(Some(&file_view));

        let image_view = ImageView::new();
        image_view.set_zoom_mode(ZoomMode::Fill);
        hbox.append(&image_view);

        let info_widget = ScrolledWindow::new();
        info_widget.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
        info_widget.set_can_focus(false);
        hbox.append(&info_widget);

        let info_view = InfoView::new();
        info_view.set_vexpand(true);
        // info_view.set_fixed_height_mode(true);
        info_view.set_can_focus(false);
        info_widget.set_child(Some(&info_view));

        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            Propagation::Stop,
            move |_ctrl, key, _, _| {
                this.on_key_press(key);
                Propagation::Stop
            }
        ));
        self.obj().add_controller(key_controller);

        let gesture_click = gtk4::GestureClick::new();
        gesture_click.set_button(1);
        gesture_click.connect_pressed(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, _n_press, x, y| this.on_mouse_press((x, y))
        ));
        image_view.add_controller(gesture_click);

        image_view.connect_closure(
            SIGNAL_VIEW_RESIZED,
            false,
            closure_local!(
                #[weak(rename_to = this)]
                self,
                move |_view: ImageView, _width: i32, _height: i32| {
                    // println!("view was resized to {_width} {_height}");
                    this.update_thumbnail_backend();
                }
            ),
        );

        file_view.connect_cursor_changed(clone!(
            #[weak(rename_to = this)]
            self,
            move |_| this.on_cursor_changed()
        ));

        file_view.connect_row_activated(clone!(
            #[weak(rename_to = this)]
            self,
            move |_, path, column| {
                this.on_row_activated(path, column);
            }
        ));

        let (sender, receiver) = async_channel::unbounded::<Message>();

        self.widget_cell
            .set(MViewWidgets {
                hbox,
                file_view,
                file_widget,
                info_widget,
                info_view,
                image_view,
                sender,
            })
            .expect("Failed to initialize MView window");

        let w = self.widgets();
        glib::spawn_future_local(clone!(
            #[strong(rename_to = image_view)]
            w.image_view,
            #[strong(rename_to = sender)]
            w.sender,
            async move {
                let mut current_task = 0;
                let mut command = TCommand::default();
                while let Ok(msg) = receiver.recv().await {
                    match msg {
                        Message::Command(cmd) => {
                            command = cmd;
                            current_task = 0;
                            if command.needs_work() {
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                            } else {
                                image_view.set_image_post();
                            }
                        }
                        Message::Result(res) => {
                            if handle_thumbnail_result(&image_view, &mut command, res) {
                                start_thumbnail_task(
                                    &sender,
                                    &image_view,
                                    &command,
                                    &mut current_task,
                                );
                            }
                        }
                    }
                }
            }
        ));

        self.show_info_widget(false);
        window.set_child(Some(&w.hbox));
        // window.show();

        self.set_backend(<dyn Backend>::current_dir(), Selection::None, false);

        println!("MViewWindow: constructed");
    }
}

impl WidgetImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}

impl MViewWidgets {
    pub fn filter(&self) -> Filter {
        if self.file_widget.is_visible() {
            Filter::None
        } else {
            Filter::Image
        }
    }
}
