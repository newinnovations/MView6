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

mod actions;
mod backend;
mod dependencies;
mod keyboard;
mod menu;
mod mouse;
mod navigate;
mod resize;
mod sort;

use crate::{
    backends::{
        document::PageMode,
        thumbnail::{
            processing::{handle_thumbnail_result, start_thumbnail_task},
            Message, TCommand,
        },
        Backend,
    },
    file_view::{
        model::{BackendRef, ItemRef, Reference},
        FileView, Sort, Target,
    },
    image::view::{ImageView, SIGNAL_CANVAS_RESIZED, SIGNAL_NAVIGATE},
    info_view::InfoView,
    rect::PointD,
    render_thread::{
        model::{RenderCommand, RenderCommandMessage, RenderReply, RenderReplyMessage},
        RenderThread, RenderThreadSender,
    },
    window::imp::dependencies::check_dependencies,
};
use async_channel::Sender;
use gio::{SimpleAction, SimpleActionGroup};
use glib::{clone, closure_local, idle_add_local, ControlFlow, SourceId};
use gtk4::{
    glib::Propagation, prelude::*, subclass::prelude::*, Button, EventControllerKey, HeaderBar,
    MenuButton, ScrolledWindow,
};
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashMap,
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug)]
pub struct MViewWidgets {
    hbox: gtk4::Box,
    file_widget: ScrolledWindow,
    file_view: FileView,
    info_widget: ScrolledWindow,
    info_view: InfoView,
    image_view: ImageView,
    pub tn_sender: Sender<Message>,
    _render_thread: RenderThread,
    pub rt_sender: RenderThreadSender,
    actions: SimpleActionGroup,
    forward_button: Button,
}

impl MViewWidgets {
    pub fn set_action_string(&self, action_name: &str, state: &str) {
        if let Some(action) = self.actions.lookup_action(action_name) {
            if let Ok(action) = action.downcast::<SimpleAction>() {
                action.set_state(&state.to_variant());
            }
        }
    }

    pub fn set_action_bool(&self, action_name: &str, state: bool) {
        if let Some(action) = self.actions.lookup_action(action_name) {
            if let Ok(action) = action.downcast::<SimpleAction>() {
                action.set_state(&state.to_variant());
            }
        }
    }

    pub fn rb_send(&self, command: RenderCommand) {
        self.rt_sender.send_blocking(command);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetTime {
    pub target: Target,
    // pub sort: Sort,
    pub timestamp: u64,
}

impl TargetTime {
    pub fn new(target: &Target) -> Self {
        TargetTime {
            target: target.clone(),
            // sort: Sort::Unsorted,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[derive(Debug, Default)]
pub struct MViewWindowImp {
    widget_cell: OnceCell<MViewWidgets>,
    backend: RefCell<Box<dyn Backend>>,
    fullscreen: Cell<bool>,
    pub skip_loading: Cell<bool>,
    pub open_container: Cell<bool>,
    thumbnail_size: Cell<i32>,
    current_sort: Cell<Sort>,
    page_mode: Cell<PageMode>,
    sorting_store: RefCell<HashMap<PathBuf, Sort>>,
    target_store: RefCell<HashMap<PathBuf, TargetTime>>,
    canvas_resized_timeout_id: RefCell<Option<SourceId>>,
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
        w.set_action_bool("pane.files", show);
        if w.file_widget.is_visible() != show {
            w.file_widget.set_visible(show);
            self.update_layout();
        }
    }

    pub fn show_info_widget(&self, show: bool) {
        let w = self.widgets();
        w.set_action_bool("pane.info", show);
        if w.info_widget.is_visible() != show {
            w.info_widget.set_visible(show);
            self.update_layout();
        }
    }

    pub fn update_layout(&self) {
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
        let backend = self.backend.borrow();
        let shrink_file_view =
            w.info_widget.is_visible() || backend.is_thumbnail() || backend.is_doc();
        w.file_view.set_extended(!shrink_file_view);
    }

    pub fn step_size(&self) -> u32 {
        if self.backend.borrow().is_doc() {
            match self.page_mode.get() {
                PageMode::Single => 1,
                PageMode::DualEvenOdd => 2,
                PageMode::DualOddEven => 2,
            }
        } else {
            1
        }
    }
}

impl ObjectImpl for MViewWindowImp {
    fn constructed(&self) {
        self.parent_constructed();

        _ = self.load_navigation();

        let args: Vec<String> = env::args().collect();
        let filename = if args.len() > 1 {
            Some(args[1].clone())
        } else {
            None
        };

        self.thumbnail_size.set(250);
        self.current_sort.set(Sort::sort_on_category());

        let window = self.obj();

        window.set_title(Some("MView6"));
        // window.set_position(gtk4::WindowPosition::Center); TODO
        window.set_default_size(1280, 720);

        // ---- Create a menu button in the window header bar

        let header_bar = HeaderBar::new();

        let back_button = Button::builder()
            .icon_name("go-previous-symbolic")
            .can_focus(false)
            .build();
        back_button.connect_clicked(clone!(
            #[weak(rename_to = this)]
            self,
            move |_button| {
                this.dir_leave();
            }
        ));
        header_bar.pack_start(&back_button);

        // Create a menu button with hamburger icon
        let menu_button = MenuButton::builder()
            .icon_name("open-menu-symbolic") // hamburger icon
            .can_focus(false)
            .build();

        let menu = Self::create_main_menu();

        // Set the menu to the button
        menu_button.set_menu_model(Some(&menu));

        // Pack the menu button at the start of the header bar
        header_bar.pack_start(&menu_button);

        let forward_button = Button::builder()
            .icon_name("go-next-symbolic")
            .can_focus(false)
            .build();
        forward_button.connect_clicked(clone!(
            #[weak(rename_to = this)]
            self,
            move |_button| {
                this.dir_enter();
            }
        ));
        header_bar.pack_start(&forward_button);

        // Set the header bar as the title bar of the window
        window.set_titlebar(Some(&header_bar));

        // Create action group for window-specific actions
        let actions = self.setup_actions();

        // Add the action group to the window
        window.insert_action_group("win", Some(&actions));

        // ----

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
            move |_, _n_press, x, y| this.on_mouse_press(PointD::new(x, y))
        ));
        image_view.add_controller(gesture_click);

        image_view.connect_closure(
            SIGNAL_CANVAS_RESIZED,
            false,
            closure_local!(
                #[weak(rename_to = this)]
                self,
                move |_view: ImageView, width: i32, height: i32| {
                    this.event_canvas_resized(width, height);
                }
            ),
        );

        image_view.connect_closure(
            SIGNAL_NAVIGATE,
            false,
            closure_local!(
                #[weak(rename_to = this)]
                self,
                move |_view: ImageView, name: &str, path: &str, item_str: &str| {
                    // dbg!(item_str);
                    // let _ = dbg!(ItemRef::from_string_repr(item_str));
                    this.event_navigate(Reference {
                        backend: BackendRef::new(name, path.into()),
                        item: ItemRef::from_string_repr(item_str).unwrap_or_default(),
                    });
                }
            ),
        );

        image_view.add_context_menu(menu);

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

        let (tn_sender, tn_receiver) = async_channel::unbounded::<Message>();
        let (to_rt_sender, to_rt_receiver) = async_channel::unbounded::<RenderCommandMessage>();
        let (from_rt_sender, from_rt_receiver) = async_channel::unbounded::<RenderReplyMessage>();

        let render_thread = RenderThread::new(from_rt_sender, to_rt_receiver);
        let rt_sender = render_thread.create_sender(to_rt_sender);

        self.widget_cell
            .set(MViewWidgets {
                hbox,
                file_view,
                file_widget,
                info_widget,
                info_view,
                image_view,
                tn_sender,
                _render_thread: render_thread,
                rt_sender,
                actions,
                forward_button,
            })
            .expect("Failed to initialize MView window");

        let w = self.widgets();

        w.image_view.init(w);

        glib::spawn_future_local(clone!(
            #[strong(rename_to = image_view)]
            w.image_view,
            #[strong(rename_to = sender)]
            w.tn_sender,
            async move {
                let mut current_task = 0;
                let mut command = TCommand::default();
                while let Ok(msg) = tn_receiver.recv().await {
                    match msg {
                        Message::Command(cmd) => {
                            command = *cmd;
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
                                // Nothing to do for the command
                                image_view.set_image_post(Default::default());
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

        glib::spawn_future_local(clone!(
            #[strong(rename_to = image_view)]
            w.image_view,
            #[strong(rename_to = _sender)]
            w.tn_sender,
            async move {
                while let Ok(msg) = from_rt_receiver.recv().await {
                    match msg.reply {
                        RenderReply::RenderDone(image_id, surface_data, zoom, viewport) => {
                            image_view.hq_render_reply(image_id, surface_data, zoom, viewport);
                            // println!("Got reply HqRender");
                        }
                    }
                }
            }
        ));

        self.show_info_widget(false);
        window.set_child(Some(&w.hbox));

        idle_add_local(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            ControlFlow::Break,
            move || {
                check_dependencies(&this.obj(), false);
                if let Some(filename) = &filename {
                    println!("Opening {filename}");
                    // match path::absolute(filename) {
                    match fs::canonicalize(filename) {
                        Ok(abs_path) => this.navigate_to(&abs_path),
                        Err(_) => this.set_backend(<dyn Backend>::current_dir(), &Target::First),
                    }
                } else {
                    this.set_backend(<dyn Backend>::current_dir(), &Target::First);
                }
                ControlFlow::Break
            }
        ));

        window.connect_close_request(clone!(
            #[weak(rename_to = this)]
            self,
            #[upgrade_or]
            Propagation::Proceed,
            move |_| {
                println!("Closing");
                let _ = this.save_navigation();
                Propagation::Proceed
            }
        ));

        // println!("MViewWindow: constructed");
    }
}

impl WidgetImpl for MViewWindowImp {}
impl WindowImpl for MViewWindowImp {}
impl ApplicationWindowImpl for MViewWindowImp {}

// impl MViewWidgets {
//     pub fn filter(&self) -> Filter {
//         if self.file_widget.is_visible() {
//             Filter::None
//         } else {
//             Filter::Image
//         }
//     }
// }
