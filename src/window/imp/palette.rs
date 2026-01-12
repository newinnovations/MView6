use glib::{subclass::types::ObjectSubclassIsExt, Propagation};
use gtk4::{
    gdk::{Key, ModifierType},
    prelude::*,
    Align, Box, Entry, EventControllerKey, Label, ListBox, ListBoxRow, Orientation, PolicyType,
    ScrolledWindow, SelectionMode, Window,
};

use crate::window::imp::MViewWindowImp;
use crate::window::MViewWindow;

const COMMANDS: &[Command] = &[
    Command {
        name: "Edit navigation filter",
        shortcut: Some("Shift+F"),
        action: |w| w.adjust_filter(),
    },
    Command {
        name: "Toggle fullscreen",
        shortcut: Some("F"),
        action: |w| w.toggle_fullscreen(),
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
    action: fn(&MViewWindowImp),
}

pub struct CommandPalette {
    window: Window,
    search_entry: Entry,
    list_box: ListBox,
}

impl CommandPalette {
    pub fn new(parent: &MViewWindow) -> Self {
        let window = Window::builder()
            .transient_for(parent)
            .modal(true)
            .default_width(600)
            .default_height(400)
            .title("MView6 Command Palette")
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
            .hscrollbar_policy(PolicyType::Never)
            .vscrollbar_policy(PolicyType::Automatic)
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

        for (idx, command) in COMMANDS.iter().enumerate() {
            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = self.create_command_row(command, idx);
                self.list_box.append(&row);
            }
        }

        // Select first item
        if let Some(first_row) = self.list_box.row_at_index(0) {
            self.list_box.select_row(Some(&first_row));
        }
    }

    fn create_command_row(&self, command: &Command, command_idx: usize) -> ListBoxRow {
        let row = ListBoxRow::new();

        // Store the command index as a string in the row's name
        row.set_widget_name(&command_idx.to_string());

        let row_box = Box::new(Orientation::Horizontal, 12);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);

        let label = Label::new(Some(command.name));
        label.set_halign(Align::Start);
        label.set_hexpand(true);

        row_box.append(&label);

        if let Some(shortcut) = &command.shortcut {
            let shortcut_label = Label::new(Some(shortcut));
            shortcut_label.add_css_class("shortcut-label");
            shortcut_label.set_halign(Align::End);
            row_box.append(&shortcut_label);
        }

        row.set_child(Some(&row_box));
        row
    }

    fn setup_signals(&mut self, parent: &MViewWindow) {
        let list_box = self.list_box.clone();
        let window = self.window.clone();
        let search_entry = self.search_entry.clone();

        // Handle search entry changes
        let list_box_clone = list_box.clone();
        self.search_entry.connect_changed(move |entry| {
            let text = entry.text();
            Self::update_list(&list_box_clone, &text);
        });

        // Handle Enter key on search entry
        let list_box_clone = list_box.clone();
        let window_clone = window.clone();
        let parent_clone = parent.clone();
        self.search_entry.connect_activate(move |_| {
            if let Some(row) = list_box_clone.selected_row() {
                // Get the command index from the row's name
                if let Ok(command_idx) = row.widget_name().parse::<usize>() {
                    if let Some(command) = COMMANDS.get(command_idx) {
                        (command.action)(parent_clone.imp());
                    }
                }
                window_clone.close();
            }
        });

        // Handle Up/Down keys on search entry
        let search_key_controller = EventControllerKey::new();
        let list_box_clone = list_box.clone();
        search_key_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                Key::Down => {
                    // Move focus to the selected row in the list
                    if let Some(row) = list_box_clone.selected_row() {
                        row.grab_focus();
                    }
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            }
        });
        self.search_entry.add_controller(search_key_controller);

        // Handle row activation
        let window_clone = window.clone();
        let parent_clone = parent.clone();
        list_box.connect_row_activated(move |_, row| {
            // Get the command index from the row's name
            if let Ok(command_idx) = row.widget_name().parse::<usize>() {
                if let Some(command) = COMMANDS.get(command_idx) {
                    (command.action)(parent_clone.imp());
                }
            }
            window_clone.close();
        });

        // Handle keys on the list box
        let list_key_controller = EventControllerKey::new();
        let search_entry_clone = search_entry.clone();
        let list_box_clone = list_box.clone();
        list_key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            match key {
                Key::Up => {
                    // If at the top of the list, move focus back to search entry
                    if let Some(row) = list_box_clone.selected_row() {
                        if row.index() == 0 {
                            search_entry_clone.grab_focus();
                            return Propagation::Stop;
                        }
                    }
                    Propagation::Proceed
                }
                Key::Escape => Propagation::Proceed,
                // For any other printable character, redirect to search entry
                _ => {
                    // Check if this is a printable character (not a modifier or special key)
                    if !modifiers.contains(ModifierType::CONTROL_MASK)
                        && !modifiers.contains(ModifierType::ALT_MASK)
                        && key != Key::Shift_L
                        && key != Key::Shift_R
                        && key != Key::Control_L
                        && key != Key::Control_R
                        && key != Key::Alt_L
                        && key != Key::Alt_R
                        && key != Key::Down
                        && key != Key::Return
                    {
                        // Get the character representation of the key
                        if let Some(ch) = key.to_unicode() {
                            let current_text = search_entry_clone.text();
                            let cursor_pos = search_entry_clone.position();

                            // Insert the character at cursor position
                            let mut new_text = current_text.to_string();
                            new_text.insert(cursor_pos as usize, ch);

                            // Grab focus first
                            search_entry_clone.grab_focus();

                            // Then set text and position
                            search_entry_clone.set_text(&new_text);
                            search_entry_clone.set_position(cursor_pos + 1);
                        }
                        Propagation::Stop
                    } else {
                        Propagation::Proceed
                    }
                }
            }
        });
        self.list_box.add_controller(list_key_controller);

        // Handle Escape key on window level
        let key_controller = EventControllerKey::new();
        let window_clone = window.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == Key::Escape {
                window_clone.close();
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
        self.window.add_controller(key_controller);
    }

    fn update_list(list_box: &ListBox, filter: &str) {
        while let Some(row) = list_box.first_child() {
            list_box.remove(&row);
        }

        let filter_lower = filter.to_lowercase();

        for (idx, command) in COMMANDS.iter().enumerate() {
            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = Self::create_row_static(command, idx);
                list_box.append(&row);
            }
        }

        if let Some(first_row) = list_box.row_at_index(0) {
            list_box.select_row(Some(&first_row));
        }
    }

    fn create_row_static(command: &Command, command_idx: usize) -> ListBoxRow {
        let row = ListBoxRow::new();

        // Store the command index as a string in the row's name
        row.set_widget_name(&command_idx.to_string());

        let row_box = Box::new(Orientation::Horizontal, 12);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);

        let label = Label::new(Some(command.name));
        label.set_halign(Align::Start);
        label.set_hexpand(true);

        row_box.append(&label);

        if let Some(shortcut) = &command.shortcut {
            let shortcut_label = Label::new(Some(shortcut));
            shortcut_label.add_css_class("shortcut-label");
            shortcut_label.set_halign(Align::End);
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
