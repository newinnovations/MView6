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

impl CommandPalette {
    pub fn new(parent: &MViewWindow, recent_commands: Rc<RefCell<VecDeque<usize>>>) -> Self {
        let window = Window::builder()
            .transient_for(parent)
            .modal(true)
            .default_width(600)
            .default_height(400)
            .title("MView6 Command Palette")
            .build();

        let main_box = Box::new(Orientation::Vertical, 0);

        let search_entry = Entry::builder()
            .placeholder_text("Type a command name...")
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(8)
            .build();

        search_entry.add_css_class("cp-command-search");

        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::Single)
            .build();

        list_box.add_css_class("cp-command-list");

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
        // Clear existing rows efficiently
        while let Some(row) = self.list_box.first_child() {
            self.list_box.remove(&row);
        }

        let filter_lower = filter.to_lowercase();
        let recent = self.recent_commands.borrow();

        // Show recent commands section if no filter
        if filter.is_empty() && !recent.is_empty() {
            Self::add_section_header(&self.list_box, "Recent");

            for &command_idx in recent.iter() {
                if let Some(command) = COMMANDS.get(command_idx) {
                    let row = Self::create_command_row(command, command_idx);
                    self.list_box.append(&row);
                }
            }

            Self::add_separator(&self.list_box);
            Self::add_section_header(&self.list_box, "All Commands");
        }

        // Add filtered commands
        for (idx, command) in COMMANDS.iter().enumerate() {
            // Skip duplicates when showing recent
            if filter.is_empty() && recent.contains(&idx) {
                continue;
            }

            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = Self::create_command_row(command, idx);
                self.list_box.append(&row);
            }
        }

        self.select_first_selectable();
    }

    fn select_first_selectable(&self) {
        let mut index = 0;
        while let Some(row) = self.list_box.row_at_index(index) {
            if row.is_selectable() {
                self.list_box.select_row(Some(&row));
                break;
            }
            index += 1;
        }
    }

    fn create_command_row(command: &Command, command_idx: usize) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.set_widget_name(&command_idx.to_string());
        row.add_css_class("cp-command");

        let row_box = Box::new(Orientation::Horizontal, 12);
        let label = Label::new(Some(command.name));
        label.set_halign(Align::Start);
        label.set_hexpand(true);

        row_box.append(&label);

        if let Some(shortcut) = &command.shortcut {
            let shortcut_label = Label::new(Some(shortcut));
            shortcut_label.add_css_class("cp-shortcut");
            shortcut_label.set_halign(Align::End);
            row_box.append(&shortcut_label);
        }

        row.set_child(Some(&row_box));
        row
    }

    fn add_to_recent(recent_commands: &Rc<RefCell<VecDeque<usize>>>, command_idx: usize) {
        let mut recent = recent_commands.borrow_mut();

        // Remove if already exists
        recent.retain(|&x| x != command_idx);

        // Add to front
        recent.push_front(command_idx);

        // Keep only MAX_RECENT_ITEMS
        recent.truncate(MAX_RECENT_ITEMS);
    }

    fn setup_signals(&mut self, parent: &MViewWindow) {
        self.setup_search_entry(parent);
        self.setup_list_box(parent);
        self.setup_window_escape();
    }

    fn setup_search_entry(&self, parent: &MViewWindow) {
        // Search text changes
        let list_box = self.list_box.clone();
        let recent_commands = self.recent_commands.clone();
        self.search_entry.connect_changed(move |entry| {
            let text = entry.text();
            Self::update_list(&list_box, &text, &recent_commands);
        });

        // Enter key activates selected command
        let window = self.window.clone();
        let list_box = self.list_box.clone();
        let recent_commands = self.recent_commands.clone();
        let parent_clone = parent.clone();
        self.search_entry.connect_activate(move |_| {
            if let Some(row) = list_box.selected_row() {
                if let Ok(command_idx) = row.widget_name().parse::<usize>() {
                    if let Some(command) = COMMANDS.get(command_idx) {
                        Self::add_to_recent(&recent_commands, command_idx);
                        (command.action)(parent_clone.imp());
                        window.close();
                    }
                }
            }
        });

        // Down arrow moves to list
        let list_box = self.list_box.clone();
        let search_key_controller = EventControllerKey::new();
        search_key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == Key::Down {
                let mut index = 0;
                while let Some(row) = list_box.row_at_index(index) {
                    if row.is_selectable() {
                        list_box.select_row(Some(&row));
                        row.grab_focus();
                        break;
                    }
                    index += 1;
                }
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
        self.search_entry.add_controller(search_key_controller);
    }

    fn setup_list_box(&self, parent: &MViewWindow) {
        // Row activation
        let window = self.window.clone();
        let recent_commands = self.recent_commands.clone();
        let parent_clone = parent.clone();
        self.list_box.connect_row_activated(move |_, row| {
            if let Ok(command_idx) = row.widget_name().parse::<usize>() {
                if let Some(command) = COMMANDS.get(command_idx) {
                    Self::add_to_recent(&recent_commands, command_idx);
                    (command.action)(parent_clone.imp());
                    window.close();
                }
            }
        });

        // Keyboard navigation in list
        let search_entry = self.search_entry.clone();
        let list_box = self.list_box.clone();
        let list_key_controller = EventControllerKey::new();

        list_key_controller.connect_key_pressed(move |_, key, _, modifiers| match key {
            Key::Down => Self::handle_down_key(&list_box),
            Key::Up => Self::handle_up_key(&list_box, &search_entry),
            Key::Escape => Propagation::Proceed,
            _ => Self::handle_char_input(key, modifiers, &search_entry),
        });
        self.list_box.add_controller(list_key_controller);
    }

    fn handle_down_key(list_box: &ListBox) -> Propagation {
        let Some(row) = list_box.selected_row() else {
            return Propagation::Proceed;
        };

        let mut i = row.index() + 1;
        while let Some(r) = list_box.row_at_index(i) {
            if r.is_selectable() {
                list_box.select_row(Some(&r));
                r.grab_focus();
                return Propagation::Stop;
            }
            i += 1;
        }
        Propagation::Stop
    }

    fn handle_up_key(list_box: &ListBox, search_entry: &Entry) -> Propagation {
        let Some(row) = list_box.selected_row() else {
            return Propagation::Proceed;
        };

        let current_idx = row.index();
        if current_idx == 0 {
            search_entry.grab_focus();
            return Propagation::Stop;
        }

        let mut i = current_idx - 1;
        loop {
            if let Some(r) = list_box.row_at_index(i) {
                if r.is_selectable() {
                    list_box.select_row(Some(&r));
                    r.grab_focus();
                    return Propagation::Stop;
                }
            }

            if i == 0 {
                break;
            }
            i -= 1;
        }

        // Scroll to top and return to search
        if let Some(first_row) = list_box.row_at_index(0) {
            first_row.set_can_focus(true);
            first_row.grab_focus();
            first_row.set_can_focus(false);
        }
        search_entry.grab_focus();
        Propagation::Stop
    }

    fn handle_char_input(key: Key, modifiers: ModifierType, search_entry: &Entry) -> Propagation {
        // Redirect printable characters to search entry
        if modifiers.contains(ModifierType::CONTROL_MASK)
            || modifiers.contains(ModifierType::ALT_MASK)
            || matches!(
                key,
                Key::Shift_L
                    | Key::Shift_R
                    | Key::Control_L
                    | Key::Control_R
                    | Key::Alt_L
                    | Key::Alt_R
                    | Key::Down
                    | Key::Return
            )
        {
            return Propagation::Proceed;
        }

        if let Some(ch) = key.to_unicode() {
            let current_text = search_entry.text();
            let cursor_pos = search_entry.position();
            let mut new_text = current_text.to_string();
            new_text.insert(cursor_pos as usize, ch);
            search_entry.grab_focus();
            search_entry.set_text(&new_text);
            search_entry.set_position(cursor_pos + 1);
            Propagation::Stop
        } else {
            Propagation::Proceed
        }
    }

    fn setup_window_escape(&self) {
        let window = self.window.clone();
        let key_controller = EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            if key == Key::Escape {
                window.close();
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
        self.window.add_controller(key_controller);
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

        if filter.is_empty() && !recent.is_empty() {
            Self::add_section_header(list_box, "Recent");

            for &command_idx in recent.iter() {
                if let Some(command) = COMMANDS.get(command_idx) {
                    let row = Self::create_command_row(command, command_idx);
                    list_box.append(&row);
                }
            }

            Self::add_separator(list_box);
            Self::add_section_header(list_box, "All Commands");
        }

        for (idx, command) in COMMANDS.iter().enumerate() {
            if filter.is_empty() && recent.contains(&idx) {
                continue;
            }

            if filter.is_empty() || command.name.to_lowercase().contains(&filter_lower) {
                let row = Self::create_command_row(command, idx);
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

    fn add_section_header(list_box: &ListBox, text: &str) {
        let header_row = ListBoxRow::new();
        header_row.set_activatable(false);
        header_row.set_selectable(false);
        header_row.set_can_focus(false);

        let header_label = Label::new(Some(text));
        header_label.set_halign(Align::Start);
        header_label.add_css_class("cp-header");
        header_row.set_child(Some(&header_label));
        list_box.append(&header_row);
    }

    fn add_separator(list_box: &ListBox) {
        let separator_row = ListBoxRow::new();
        separator_row.set_activatable(false);
        separator_row.set_selectable(false);
        separator_row.set_can_focus(false);

        let separator = Separator::new(Orientation::Horizontal);
        separator.add_css_class("cp-separator");
        separator_row.set_child(Some(&separator));
        list_box.append(&separator_row);
    }

    pub fn show(&self) {
        self.populate_list("");
        self.search_entry.set_text("");
        self.search_entry.grab_focus();
        self.window.present();
    }
}
