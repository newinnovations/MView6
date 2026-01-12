use gtk4::prelude::*;
use gtk4::{
    glib, Box, Entry, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow, SelectionMode,
};

use crate::window::MViewWindow;

const COMMANDS: &[Command] = &[
    Command {
        name: "Clear buffer",
        shortcut: Some("Ctrl+Shift+K"),
        action: |_| println!("Clear buffer"),
    },
    Command {
        name: "Close all other panes",
        shortcut: None,
        action: |_| println!("Close all other panes"),
    },
    Command {
        name: "Close all other tabs",
        shortcut: None,
        action: |_| println!("Close all other tabs"),
    },
    Command {
        name: "Close all tabs after the current tab",
        shortcut: None,
        action: |_| println!("Close all tabs after current"),
    },
    Command {
        name: "Close pane",
        shortcut: Some("Ctrl+Shift+W"),
        action: |_| println!("Close pane"),
    },
    Command {
        name: "Close window",
        shortcut: Some("Alt+F4"),
        action: |_| println!("Close window"),
    },
    Command {
        name: "Copy text",
        shortcut: Some("Ctrl+C"),
        action: |_| println!("Copy text"),
    },
    Command {
        name: "Decrease font size",
        shortcut: Some("Ctrl+Minus"),
        action: |_| println!("Decrease font size"),
    },
    Command {
        name: "Disable pane read-only mode",
        shortcut: None,
        action: |_| println!("Disable pane read-only mode"),
    },
    Command {
        name: "Duplicate pane",
        shortcut: Some("Alt+Shift+D"),
        action: |_| println!("Duplicate pane"),
    },
];

#[derive(Clone)]
struct Command {
    name: &'static str,
    shortcut: Option<&'static str>,
    action: fn(&MViewWindow),
}

pub struct CommandPalette {
    window: gtk4::Window,
    search_entry: Entry,
    list_box: ListBox,
}

impl CommandPalette {
    pub fn new(parent: &MViewWindow) -> Self {
        let window = gtk4::Window::builder()
            .transient_for(parent)
            .modal(true)
            .default_width(600)
            .default_height(400)
            .build();

        // Main container
        let main_box = Box::new(Orientation::Vertical, 0);

        // Search entry at the top
        let search_entry = Entry::builder()
            .placeholder_text("Type a command name...")
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(8)
            .build();

        // Add CSS for the entry
        search_entry.add_css_class("command-search");

        // ListBox for commands
        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .build();

        list_box.add_css_class("command-list");

        // Scrolled window
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .vexpand(true)
            .build();

        scrolled.set_child(Some(&list_box));

        main_box.append(&search_entry);
        main_box.append(&scrolled);
        window.set_child(Some(&main_box));

        let mut palette = Self {
            window,
            search_entry,
            list_box,
        };

        palette.populate_list("");
        palette.setup_signals(parent);

        palette
    }

    fn populate_list(&self, filter: &str) {
        // Clear existing rows
        while let Some(row) = self.list_box.first_child() {
            self.list_box.remove(&row);
        }

        let filter_lower = filter.to_lowercase();

        for command in COMMANDS {
            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = self.create_command_row(command);
                self.list_box.append(&row);
            }
        }

        // Select first item
        if let Some(first_row) = self.list_box.row_at_index(0) {
            self.list_box.select_row(Some(&first_row));
        }
    }

    fn create_command_row(&self, command: &Command) -> ListBoxRow {
        let row = ListBoxRow::new();
        let row_box = Box::new(Orientation::Horizontal, 12);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);

        let label = Label::new(Some(command.name));
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);

        row_box.append(&label);

        if let Some(shortcut) = &command.shortcut {
            let shortcut_label = Label::new(Some(shortcut));
            shortcut_label.add_css_class("shortcut-label");
            shortcut_label.set_halign(gtk4::Align::End);
            row_box.append(&shortcut_label);
        }

        row.set_child(Some(&row_box));
        row
    }

    fn setup_signals(&mut self, parent: &MViewWindow) {
        let list_box = self.list_box.clone();
        let window = self.window.clone();

        // Handle search entry changes
        let list_box_clone = list_box.clone();
        self.search_entry.connect_changed(move |entry| {
            let text = entry.text();
            Self::update_list(&list_box_clone, &text);
        });

        // Handle Enter key
        let list_box_clone = list_box.clone();
        let window_clone = window.clone();
        let parent_clone = parent.clone();
        self.search_entry.connect_activate(move |_| {
            if let Some(row) = list_box_clone.selected_row() {
                let index = row.index() as usize;
                if let Some(command) = COMMANDS.get(index) {
                    (command.action)(&parent_clone);
                }
                window_clone.close();
            }
        });

        // Handle row activation
        let window_clone = window.clone();
        let parent_clone = parent.clone();
        list_box.connect_row_activated(move |_, row| {
            let index = row.index() as usize;
            if let Some(command) = COMMANDS.get(index) {
                (command.action)(&parent_clone);
            }
            window_clone.close();
        });

        // Handle Escape key
        let key_controller = gtk4::EventControllerKey::new();
        let window_clone = window.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == gtk4::gdk::Key::Escape {
                window_clone.close();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        self.window.add_controller(key_controller);
    }

    fn update_list(list_box: &ListBox, filter: &str) {
        while let Some(row) = list_box.first_child() {
            list_box.remove(&row);
        }

        let filter_lower = filter.to_lowercase();

        for command in COMMANDS {
            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = Self::create_row_static(command);
                list_box.append(&row);
            }
        }

        if let Some(first_row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&first_row));
        }
    }

    fn create_row_static(command: &Command) -> ListBoxRow {
        let row = ListBoxRow::new();
        let row_box = Box::new(Orientation::Horizontal, 12);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);

        let label = Label::new(Some(command.name));
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);

        row_box.append(&label);

        if let Some(shortcut) = &command.shortcut {
            let shortcut_label = Label::new(Some(shortcut));
            shortcut_label.add_css_class("shortcut-label");
            shortcut_label.set_halign(gtk4::Align::End);
            row_box.append(&shortcut_label);
        }

        row.set_child(Some(&row_box));
        row
    }

    pub fn show(&self) {
        self.search_entry.grab_focus();
        self.window.present();
    }
}
