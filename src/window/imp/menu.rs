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

use gio::{prelude::ActionMapExt, Menu, SimpleAction, SimpleActionGroup};
use glib::VariantTy;

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn create_main_menu() -> Menu {
        // Create the main menu
        let main_menu = Menu::new();

        let top_section = Menu::new();
        top_section.append(Some("Open"), Some("win.open"));

        let zoom_submenu = Menu::new();
        zoom_submenu.append(Some("No scaling"), Some("win.zoom::nozoom"));
        zoom_submenu.append(Some("Fit window"), Some("win.zoom::fit"));
        zoom_submenu.append(Some("Fill window"), Some("win.zoom::fill"));
        zoom_submenu.append(Some("Maximum zoom"), Some("win.zoom::max"));

        let rotate_submenu = Menu::new();
        rotate_submenu.append(Some("90° Clockwise"), Some("win.rotate::270"));
        rotate_submenu.append(Some("90° Counterclockwise"), Some("win.rotate::90"));
        rotate_submenu.append(Some("Rotate 180°"), Some("win.rotate::180"));

        let page_submenu = Menu::new();
        page_submenu.append(Some("Single"), Some("win.page::single"));
        page_submenu.append(Some("Dual (1, 2-3, 4-5, ...)"), Some("win.page::deo"));
        page_submenu.append(Some("Dual (1-2, 3-4, 5-6, ...)"), Some("win.page::doe"));

        let panes_submenu = Menu::new();
        panes_submenu.append(Some("Files"), Some("win.pane.files"));
        panes_submenu.append(Some("Information"), Some("win.pane.info"));

        let thumbnail_size_submenu = Menu::new();
        thumbnail_size_submenu.append(Some("Extra small (80 px)"), Some("win.thumb.size::80"));
        thumbnail_size_submenu.append(Some("Small (100 px)"), Some("win.thumb.size::100"));
        thumbnail_size_submenu.append(Some("Medium (140 px)"), Some("win.thumb.size::140"));
        thumbnail_size_submenu.append(Some("Large (175 px)"), Some("win.thumb.size::175"));
        thumbnail_size_submenu.append(Some("Extra large (250 px)"), Some("win.thumb.size::250"));

        let thumbnail_submenu = Menu::new();
        thumbnail_submenu.append(Some("Show thumbnails"), Some("win.thumb.show"));
        thumbnail_submenu.append_section(Some("Size"), &thumbnail_size_submenu);

        let flag_section = Menu::new();
        flag_section.append(Some("Full screen"), Some("win.fullscreen"));
        flag_section.append_submenu(Some("Thumbnails"), &thumbnail_submenu);
        flag_section.append_submenu(Some("Rotate"), &rotate_submenu);
        flag_section.append_submenu(Some("Zoom"), &zoom_submenu);
        flag_section.append_submenu(Some("Page mode"), &page_submenu);
        flag_section.append_submenu(Some("Panes"), &panes_submenu);

        let bottom_section = Menu::new();
        bottom_section.append(Some("About"), Some("win.about"));
        bottom_section.append(Some("Help"), Some("win.help"));
        bottom_section.append(Some("Quit"), Some("win.quit"));

        main_menu.append_section(None, &top_section);
        main_menu.append_section(None, &flag_section);
        main_menu.append_section(None, &bottom_section);

        main_menu
    }

    pub fn setup_actions(&self) -> SimpleActionGroup {
        let action_group = SimpleActionGroup::new();
        self.add_action(&action_group, "open", Self::open_file);
        self.add_action(&action_group, "about", Self::show_about_dialog);
        self.add_action(&action_group, "help", Self::show_help);
        self.add_action(&action_group, "quit", Self::quit);
        self.add_action_bool(&action_group, "fullscreen", false, Self::toggle_fullscreen);
        self.add_action_int(&action_group, "rotate", 0, Self::rotate_image);
        self.add_action_string(&action_group, "zoom", "fill", Self::change_zoom);
        self.add_action_string(&action_group, "page", "deo", Self::change_page_mode);
        self.add_action_bool(&action_group, "pane.files", true, Self::toggle_pane_files);
        self.add_action_bool(&action_group, "pane.info", false, Self::toggle_pane_info);
        self.add_action_bool(
            &action_group,
            "thumb.show",
            false,
            Self::toggle_thumbnail_view,
        );
        self.add_action_int(&action_group, "thumb.size", 250, Self::set_thumbnail_size);
        action_group
    }

    fn add_action<F: Fn(&MViewWindowImp) + 'static>(
        &self,
        action_group: &SimpleActionGroup,
        name: &str,
        callback: F,
    ) {
        let action = SimpleAction::new(name, None);
        let window_weak = self.downgrade();
        action.connect_activate(move |_, _| {
            if let Some(this) = window_weak.upgrade() {
                callback(&this);
            }
        });
        action_group.add_action(&action);
    }

    fn add_action_int<F: Fn(&MViewWindowImp, i32) + 'static>(
        &self,
        action_group: &SimpleActionGroup,
        name: &str,
        default: i32,
        callback: F,
    ) {
        // let action = SimpleAction::new(name, Some(VariantTy::STRING));
        let default = default.to_string();
        let action = SimpleAction::new_stateful(name, Some(VariantTy::STRING), &default.into());
        let window_weak = self.downgrade();
        action.connect_activate(move |_, param| {
            if let Some(this) = window_weak.upgrade() {
                if let Some(param) = param {
                    if let Some(text) = param.get::<String>() {
                        if let Ok(i) = text.parse::<i32>() {
                            callback(&this, i);
                        }
                    }
                }
            }
        });
        action_group.add_action(&action);
    }

    fn add_action_bool<F: Fn(&MViewWindowImp) + 'static>(
        &self,
        action_group: &SimpleActionGroup,
        name: &str,
        default: bool,
        callback: F,
    ) {
        let action = SimpleAction::new_stateful(name, None, &default.into());
        let window_weak = self.downgrade();
        action.connect_activate(move |_, _| {
            if let Some(this) = window_weak.upgrade() {
                callback(&this);
            }
        });
        action_group.add_action(&action);
    }

    fn add_action_string<F: Fn(&MViewWindowImp, &str) + 'static>(
        &self,
        action_group: &SimpleActionGroup,
        name: &str,
        default: &str,
        callback: F,
    ) {
        let action = SimpleAction::new_stateful(name, Some(VariantTy::STRING), &default.into());
        let window_weak = self.downgrade();
        action.connect_activate(move |_, param| {
            if let Some(this) = window_weak.upgrade() {
                if let Some(param) = param {
                    if let Some(text) = param.get::<String>() {
                        callback(&this, &text);
                    }
                }
            }
        });
        action_group.add_action(&action);
    }
}
