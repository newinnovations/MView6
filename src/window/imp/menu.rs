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

use chrono::Datelike;
use gio::{prelude::ActionMapExt, Menu, SimpleAction, SimpleActionGroup};
use glib::object::ObjectExt;
use gtk4::{prelude::GtkWindowExt, AboutDialog, License};

use crate::window::MViewWindow;

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn create_main_menu() -> Menu {
        // Create the main menu
        let main_menu = Menu::new();

        // File section
        let file_section = Menu::new();
        file_section.append(Some("New"), Some("win.new"));
        file_section.append(Some("Open"), Some("win.open"));
        file_section.append(Some("Save"), Some("win.save"));
        file_section.append(Some("Save As..."), Some("win.save_as"));

        // Edit section
        let edit_section = Menu::new();
        edit_section.append(Some("Cut"), Some("win.cut"));
        edit_section.append(Some("Copy"), Some("win.copy"));
        edit_section.append(Some("Paste"), Some("win.paste"));

        // View submenu
        let view_submenu = Menu::new();
        view_submenu.append(Some("Full Screen"), Some("win.fullscreen"));
        view_submenu.append(Some("Zoom In"), Some("win.zoom_in"));
        view_submenu.append(Some("Zoom Out"), Some("win.zoom_out"));
        view_submenu.append(Some("Reset Zoom"), Some("win.zoom_reset"));

        // Settings and Help section
        let settings_section = Menu::new();
        settings_section.append(Some("Preferences"), Some("win.preferences"));

        // Add all sections to the main menu
        // First file section
        main_menu.append_section(Some("File"), &file_section);

        // Edit section with separator
        main_menu.append_section(Some("Edit"), &edit_section);

        // View submenu
        main_menu.append_submenu(Some("View"), &view_submenu);

        // Settings section with separator
        main_menu.append_section(Some("Settings"), &settings_section);

        // Append quit directly to main menu as its own section
        let quit_section = Menu::new();
        quit_section.append(Some("About"), Some("win.about"));
        quit_section.append(Some("Quit"), Some("win.quit"));
        main_menu.append_section(None, &quit_section);

        main_menu
    }

    pub fn setup_actions(window: &MViewWindow, action_group: &SimpleActionGroup) {
        // File actions
        Self::add_simple_action(action_group, "new", || println!("New file"));
        Self::add_simple_action(action_group, "open", || println!("Open file"));
        Self::add_simple_action(action_group, "save", || println!("Save file"));
        Self::add_simple_action(action_group, "save_as", || println!("Save as..."));

        // Edit actions
        Self::add_simple_action(action_group, "cut", || println!("Cut"));
        Self::add_simple_action(action_group, "copy", || println!("Copy"));
        Self::add_simple_action(action_group, "paste", || println!("Paste"));

        // View actions
        Self::add_simple_action(action_group, "fullscreen", || println!("Toggle fullscreen"));
        Self::add_simple_action(action_group, "zoom_in", || println!("Zoom in"));
        Self::add_simple_action(action_group, "zoom_out", || println!("Zoom out"));
        Self::add_simple_action(action_group, "zoom_reset", || println!("Reset zoom"));

        // Settings actions
        Self::add_simple_action(action_group, "preferences", || println!("Preferences"));

        // About action
        let window_clone = window.clone();
        let action_about = SimpleAction::new("about", None);
        action_about.connect_activate(move |_, _| {
            Self::show_about_dialog(&window_clone);
        });

        action_group.add_action(&action_about);
        // Quit action
        let window_weak = window.downgrade();
        let action_quit = SimpleAction::new("quit", None);
        action_quit.connect_activate(move |_, _| {
            if let Some(w) = window_weak.upgrade() {
                w.close();
            }
        });
        action_group.add_action(&action_quit);
    }

    fn add_simple_action<F: Fn() + 'static>(
        action_group: &SimpleActionGroup,
        name: &str,
        callback: F,
    ) {
        let action = SimpleAction::new(name, None);
        action.connect_activate(move |_, _| {
            callback();
        });
        action_group.add_action(&action);
    }

    fn show_about_dialog(parent: &MViewWindow) {
        let dialog = AboutDialog::builder()
            .transient_for(parent)
            .modal(true)
            .program_name("MView6")
            .version(env!("CARGO_PKG_VERSION")) // Get version from Cargo.toml
            .logo_icon_name("mview6") // This will load the icon from resources
            .authors(vec![env!("CARGO_PKG_AUTHORS")]) // Get authors from Cargo.toml
            .copyright(format!(
                "© {} {}",
                chrono::Local::now().year(),
                env!("CARGO_PKG_AUTHORS")
            ))
            .comments(env!("CARGO_PKG_DESCRIPTION"))
            .license_type(License::Agpl30)
            .website(env!("CARGO_PKG_REPOSITORY")) // Get repository URL from Cargo.toml
            .website_label("Visit source repository")
            .build();
        dialog.present();
    }
}
