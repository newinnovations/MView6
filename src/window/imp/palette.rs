use glib::{subclass::types::ObjectSubclassIsExt, Propagation};
use gtk4::{
    gdk::{Key, ModifierType},
    prelude::*,
    Align, Box, Entry, EventControllerKey, Label, ListBox, ListBoxRow, Orientation, PolicyType,
    ScrolledWindow, SelectionMode, Separator, Window,
};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::window::imp::commands::{Command, COMMANDS};
use crate::window::MViewWindow;

const MAX_RECENT_ITEMS: usize = 4;

pub struct CommandPalette {
    window: Window,
    search_entry: Entry,
    list_box: ListBox,
    recent_commands: Rc<RefCell<VecDeque<usize>>>,
}

// Would it be better using ListView or ColumnView instead of ListBox? We are targeting gtk 4.6 at the moment.

impl CommandPalette {
    pub fn new(parent: &MViewWindow, recent_commands: Rc<RefCell<VecDeque<usize>>>) -> Self {
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
            recent_commands,
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
        let recent = self.recent_commands.borrow();

        // If no filter, show recent items first
        if filter.is_empty() && !recent.is_empty() {
            // Add recent section header
            let header_row = ListBoxRow::new();
            header_row.set_activatable(false);
            header_row.set_selectable(false);
            header_row.set_can_focus(false);

            let header_label = Label::new(Some("Recent"));
            header_label.set_halign(Align::Start);
            header_label.add_css_class("dim-label");
            header_label.set_margin_start(12);
            header_label.set_margin_end(12);
            header_label.set_margin_top(8);
            header_label.set_margin_bottom(4);

            header_row.set_child(Some(&header_label));
            self.list_box.append(&header_row);

            // Add recent commands
            for &command_idx in recent.iter() {
                if let Some(command) = COMMANDS.get(command_idx) {
                    let row = self.create_command_row(command, command_idx);
                    self.list_box.append(&row);
                }
            }

            // Add separator
            let separator_row = ListBoxRow::new();
            separator_row.set_activatable(false);
            separator_row.set_selectable(false);
            separator_row.set_can_focus(false);

            let separator = Separator::new(Orientation::Horizontal);
            separator.set_margin_start(12);
            separator.set_margin_end(12);
            separator.set_margin_top(8);
            separator.set_margin_bottom(8);

            separator_row.set_child(Some(&separator));
            self.list_box.append(&separator_row);

            // Add "All Commands" header
            let all_header_row = ListBoxRow::new();
            all_header_row.set_activatable(false);
            all_header_row.set_selectable(false);
            all_header_row.set_can_focus(false);

            let all_header_label = Label::new(Some("All Commands"));
            all_header_label.set_halign(Align::Start);
            all_header_label.add_css_class("dim-label");
            all_header_label.set_margin_start(12);
            all_header_label.set_margin_end(12);
            all_header_label.set_margin_top(4);
            all_header_label.set_margin_bottom(4);

            all_header_row.set_child(Some(&all_header_label));
            self.list_box.append(&all_header_row);
        }

        // Add all commands (or filtered commands)
        for (idx, command) in COMMANDS.iter().enumerate() {
            // Skip recent commands if no filter
            if filter.is_empty() && recent.contains(&idx) {
                continue;
            }

            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = self.create_command_row(command, idx);
                self.list_box.append(&row);
            }
        }

        // Select first selectable item
        let mut index = 0;
        while let Some(row) = self.list_box.row_at_index(index) {
            if row.is_selectable() {
                self.list_box.select_row(Some(&row));
                break;
            }
            index += 1;
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
        let recent_commands = self.recent_commands.clone();

        // Handle search entry changes
        let list_box_clone = list_box.clone();
        let recent_clone = recent_commands.clone();
        self.search_entry.connect_changed(move |entry| {
            let text = entry.text();
            Self::update_list(&list_box_clone, &text, &recent_clone);
        });

        // Handle Enter key on search entry
        let list_box_clone = list_box.clone();
        let window_clone = window.clone();
        let parent_clone = parent.clone();
        let recent_clone = recent_commands.clone();
        self.search_entry.connect_activate(move |_| {
            if let Some(row) = list_box_clone.selected_row() {
                // Get the command index from the row's name
                if let Ok(command_idx) = row.widget_name().parse::<usize>() {
                    if let Some(command) = COMMANDS.get(command_idx) {
                        Self::add_to_recent_static(&recent_clone, command_idx);
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
                    // Move focus to the first selectable row in the list
                    let mut index = 0;
                    while let Some(row) = list_box_clone.row_at_index(index) {
                        if row.is_selectable() {
                            list_box_clone.select_row(Some(&row));
                            row.grab_focus();
                            break;
                        }
                        index += 1;
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
        let recent_clone = recent_commands.clone();
        list_box.connect_row_activated(move |_, row| {
            // Get the command index from the row's name
            if let Ok(command_idx) = row.widget_name().parse::<usize>() {
                if let Some(command) = COMMANDS.get(command_idx) {
                    Self::add_to_recent_static(&recent_clone, command_idx);
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
                Key::Down => {
                    if let Some(row) = list_box_clone.selected_row() {
                        let mut i = row.index() + 1;
                        while let Some(r) = list_box_clone.row_at_index(i) {
                            if r.is_selectable() {
                                list_box_clone.select_row(Some(&r));
                                r.grab_focus();
                                return Propagation::Stop;
                            }
                            i += 1;
                        }
                        // Nothing selectable below, swallow to avoid landing on non-selectables
                        return Propagation::Stop;
                    }
                    Propagation::Proceed
                }
                Key::Up => {
                    if let Some(row) = list_box_clone.selected_row() {
                        // Walk upwards to the previous selectable row
                        let mut i = row.index();
                        if i == 0 {
                            // No rows above — go back to search
                            search_entry_clone.grab_focus();
                            return Propagation::Stop;
                        }
                        i -= 1;
                        while let Some(r) = list_box_clone.row_at_index(i) {
                            if r.is_selectable() {
                                list_box_clone.select_row(Some(&r));
                                r.grab_focus();
                                return Propagation::Stop;
                            }
                            if i == 0 {
                                break;
                            }
                            i -= 1;
                        }
                        // If we didn’t find a selectable above, return to the search entry
                        if let Some(first_row) = list_box_clone.row_at_index(0) {
                            // Hack to move the "Recent" header scroll to the first row
                            // at the top of the viewport to make it visible again
                            first_row.set_can_focus(true);
                            first_row.grab_focus();
                            first_row.set_can_focus(false);
                        }
                        search_entry_clone.grab_focus();
                        return Propagation::Stop;
                    }
                    Propagation::Proceed
                }
                Key::Escape => Propagation::Proceed,
                _ => {
                    // existing printable-char-to-search-entry redirection stays as-is
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
                        if let Some(ch) = key.to_unicode() {
                            let current_text = search_entry_clone.text();
                            let cursor_pos = search_entry_clone.position();
                            let mut new_text = current_text.to_string();
                            new_text.insert(cursor_pos as usize, ch);
                            search_entry_clone.grab_focus();
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

    fn add_to_recent_static(recent_commands: &Rc<RefCell<VecDeque<usize>>>, command_idx: usize) {
        let mut recent = recent_commands.borrow_mut();

        // Remove if already exists
        if let Some(pos) = recent.iter().position(|&x| x == command_idx) {
            recent.remove(pos);
        }

        // Add to front
        recent.push_front(command_idx);

        // Keep only MAX_RECENT_ITEMS
        while recent.len() > MAX_RECENT_ITEMS {
            recent.pop_back();
        }
    }

    fn update_list(
        list_box: &ListBox,
        filter: &str,
        recent_commands: &Rc<RefCell<VecDeque<usize>>>,
    ) {
        while let Some(row) = list_box.first_child() {
            list_box.remove(&row);
        }

        let filter_lower = filter.to_lowercase();
        let recent = recent_commands.borrow();

        // If no filter, show recent items first
        if filter.is_empty() && !recent.is_empty() {
            // Add recent section header
            let header_row = ListBoxRow::new();
            header_row.set_activatable(false);
            header_row.set_selectable(false);

            let header_label = Label::new(Some("Recent"));
            header_label.set_halign(Align::Start);
            header_label.add_css_class("dim-label");
            header_label.set_margin_start(12);
            header_label.set_margin_end(12);
            header_label.set_margin_top(8);
            header_label.set_margin_bottom(4);

            header_row.set_child(Some(&header_label));
            list_box.append(&header_row);

            // Add recent commands
            for &command_idx in recent.iter() {
                if let Some(command) = COMMANDS.get(command_idx) {
                    let row = Self::create_row_static(command, command_idx);
                    list_box.append(&row);
                }
            }

            // Add separator
            let separator_row = ListBoxRow::new();
            separator_row.set_activatable(false);
            separator_row.set_selectable(false);

            let separator = Separator::new(Orientation::Horizontal);
            separator.set_margin_start(12);
            separator.set_margin_end(12);
            separator.set_margin_top(8);
            separator.set_margin_bottom(8);

            separator_row.set_child(Some(&separator));
            list_box.append(&separator_row);

            // Add "All Commands" header
            let all_header_row = ListBoxRow::new();
            all_header_row.set_activatable(false);
            all_header_row.set_selectable(false);

            let all_header_label = Label::new(Some("All Commands"));
            all_header_label.set_halign(Align::Start);
            all_header_label.add_css_class("dim-label");
            all_header_label.set_margin_start(12);
            all_header_label.set_margin_end(12);
            all_header_label.set_margin_top(4);
            all_header_label.set_margin_bottom(4);

            all_header_row.set_child(Some(&all_header_label));
            list_box.append(&all_header_row);
        }

        for (idx, command) in COMMANDS.iter().enumerate() {
            // Skip recent commands if no filter
            if filter.is_empty() && recent.contains(&idx) {
                continue;
            }

            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = Self::create_row_static(command, idx);
                list_box.append(&row);
            }
        }

        // Select first selectable item
        let mut index = 0;
        while let Some(row) = list_box.row_at_index(index) {
            if row.is_selectable() {
                list_box.select_row(Some(&row));
                break;
            }
            index += 1;
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
        // Refresh the list to show current recent items
        self.populate_list("");
        self.search_entry.set_text("");
        self.search_entry.grab_focus();
        self.window.present();
    }
}
