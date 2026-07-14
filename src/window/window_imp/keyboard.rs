// MView6 -- High-performance PDF and photo viewer built with Rust and GTK4
//
// Copyright (c) 2024-2026 Martin van der Werff <github (at) newinnovations.nl>
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

use std::fs::File;

use super::MViewWindowImp;

use glib::subclass::types::{ObjectSubclassExt, ObjectSubclassIsExt};
use gtk4::{
    gdk::{Key, ModifierType},
    prelude::{GtkWindowExt, WidgetExt},
};

use crate::{
    backends::{Backend, ImageParams, PageMode},
    config::{contrast, contrast_delta},
    content::{Content, ContentData},
    file_view::{Column, Direction, Filter, Target},
    image::ZoomMode,
    util::error_dialog,
    window::CommandPalette,
};

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, key: Key, modifiers: ModifierType) {
        let w = self.widgets();

        // While a "move to trash" toast is visible, ESC/Enter/Space cancel it
        // instead of performing their usual action.
        if matches!(key, Key::Escape | Key::Return | Key::KP_Enter | Key::space)
            && self.has_pending_trash()
        {
            self.undo_pending_trash();
            return;
        }

        // PREV = Up, Left, z, KP_8, KP_4, KP_Up, KP_Left
        let prev = matches!(
            key,
            Key::Up
                | Key::Left
                | Key::z
                | Key::Z
                | Key::KP_Up
                | Key::KP_8
                | Key::KP_Left
                | Key::KP_4
        );

        // NEXT = Down, Right, x, KP_2, KP_6, KP_Down, KP_Right
        let next = matches!(
            key,
            Key::Down
                | Key::Right
                | Key::x
                | Key::X
                | Key::KP_Down
                | Key::KP_2
                | Key::KP_Right
                | Key::KP_6
        );

        // dbg!(key, modifiers, prev, next);

        if prev || next {
            let direction = if prev { Direction::Up } else { Direction::Down };

            if modifiers.contains(ModifierType::ALT_MASK) {
                self.hop(direction);
                return;
            }

            let count = if modifiers.contains(ModifierType::CONTROL_MASK) {
                self.step_size() * 5
            } else {
                self.step_size()
            };

            let ignore_filter = modifiers.contains(ModifierType::SHIFT_MASK)
                || !self.backend.borrow().supports_filter();

            if ignore_filter {
                w.file_view.navigate_item(direction, &Filter::None, count);
            } else {
                w.file_view
                    .navigate_item(direction, &self.current_filter.borrow(), count);
            }

            return;
        }

        if matches!(key, Key::w | Key::e) {
            let direction = if key == Key::w {
                Direction::Up
            } else {
                Direction::Down
            };

            let count = if modifiers.contains(ModifierType::CONTROL_MASK) {
                5
            } else {
                1
            };

            w.image_view.navigate_page(direction, count);

            return;
        }

        if key == Key::Page_Up || key == Key::Page_Down {
            let direction = if key == Key::Page_Up {
                Direction::Up
            } else {
                Direction::Down
            };

            if modifiers.contains(ModifierType::ALT_MASK) {
                self.hop(direction);
                return;
            }

            let (count, ignore_filter) = if self.backend.borrow().supports_filter() {
                // filesystem or archive backend, large step sizes
                let ignore_filter = modifiers.contains(ModifierType::SHIFT_MASK);
                if modifiers.contains(ModifierType::CONTROL_MASK) {
                    (50, ignore_filter)
                } else {
                    (20, ignore_filter)
                }
            } else {
                // document backend, small step size, ignore filter
                if modifiers.contains(ModifierType::CONTROL_MASK) {
                    (self.step_size() * 10, true)
                } else {
                    (self.step_size(), true)
                }
            };

            if ignore_filter {
                w.file_view.navigate_item(direction, &Filter::None, count);
            } else {
                w.file_view
                    .navigate_item(direction, &self.current_filter.borrow(), count);
            }

            return;
        }

        if key == Key::KP_7 || key == Key::KP_Home {
            self.hop(Direction::Up);
            return;
        }

        if key == Key::KP_9 || key == Key::KP_Page_Up {
            self.hop(Direction::Down);
            return;
        }

        match key {
            Key::q => {
                self.quit();
            }
            Key::h => {
                self.show_help();
            }
            Key::d => {
                self.show_files_widget(true);
                if !self.backend.borrow().is_bookmarks() {
                    let backend = self.backend.replace(<dyn Backend>::none());
                    let target = if let Some((file_row, _)) = w.file_view.selected() {
                        backend.reference(&file_row).into()
                    } else {
                        Target::First
                    };
                    self.set_backend(
                        <dyn Backend>::bookmarks(backend, target),
                        &Target::First,
                        true,
                    );
                }
            }
            Key::C => {
                if modifiers.contains(ModifierType::CONTROL_MASK) {
                    if let Some(clipboard) = self.clipboard.borrow().as_ref() {
                        w.image_view.copy_visible_to_clipboard(clipboard);
                    }
                }
            }
            Key::c => {
                if modifiers.contains(ModifierType::CONTROL_MASK) {
                    if let Some(clipboard) = self.clipboard.borrow().as_ref() {
                        w.image_view.copy_image_to_clipboard(clipboard);
                    }
                } else {
                    self.create_preview();
                }
            }
            Key::v => {
                if modifiers.contains(ModifierType::CONTROL_MASK) {
                    if let Some(clipboard) = self.clipboard.borrow().as_ref() {
                        self.paste_image_from_clipboard(clipboard);
                    }
                }
            }
            Key::t => {
                self.toggle_thumbnail_view();
            }
            Key::w | Key::KP_7 | Key::KP_Home => {
                self.hop(Direction::Up);
            }
            Key::e | Key::KP_9 | Key::KP_Page_Up => {
                self.hop(Direction::Down);
            }
            Key::space | Key::KP_Divide => {
                self.toggle_pane_files();
            }
            Key::i => {
                self.toggle_pane_info();
            }
            Key::f | Key::KP_Multiply => {
                self.toggle_fullscreen();
            }
            Key::F => {
                self.filter_dialog(None);
            }
            Key::I | Key::V | Key::D | Key::A | Key::E => {
                self.filter_dialog(Some(key));
            }
            Key::Escape => {
                self.obj().unfullscreen();
                self.fullscreen.set(false);
                self.widgets().set_action_bool("fullscreen", false);
                w.image_view.measure_enable(false);
            }
            Key::Delete => {
                self.delete_current_file(modifiers.contains(ModifierType::SHIFT_MASK));
            }
            Key::r => {
                self.rotate_image(270);
            }
            Key::R => {
                self.rotate_image(90);
            }
            Key::Return | Key::KP_Enter => {
                self.dir_enter();
            }
            Key::BackSpace | Key::KP_Delete | Key::KP_Decimal => {
                self.dir_leave();
            }
            Key::n => {
                if w.image_view.zoom_mode() == ZoomMode::Fit {
                    self.change_zoom(ZoomMode::NoZoom.into());
                } else {
                    self.change_zoom(ZoomMode::Fit.into());
                }
            }
            Key::m | Key::KP_0 | Key::KP_Insert => {
                self.toggle_zoom();
            }
            Key::minus | Key::KP_Subtract => {
                if let Some((file_row, _)) = w.file_view.selected() {
                    if self
                        .backend
                        .borrow()
                        .set_preference(&file_row, Direction::Down)
                    {
                        w.file_view
                            .navigate_item(Direction::Down, &Filter::Image, 1);
                    }
                }
            }
            Key::equal | Key::KP_Add => {
                if let Some((file_row, _)) = w.file_view.selected() {
                    if self
                        .backend
                        .borrow()
                        .set_preference(&file_row, Direction::Up)
                    {
                        w.file_view
                            .navigate_item(Direction::Down, &Filter::Image, 1);
                    }
                }
            }
            Key::a => {
                w.file_view.navigate_item(Direction::Up, &Filter::Liked, 1);
            }
            Key::s => {
                if modifiers.contains(ModifierType::CONTROL_MASK) {
                    // Ctrl+S: save full raster image data to a file (only supported for raster images)
                    self.save_raster_data_to_file();
                    // // self.save_image_dialog();
                    // let _surface = match w.image_view.imp().visible_to_surface() {
                    //     Ok(surface) => surface,
                    //     Err(e) => {
                    //         error_dialog(
                    //             &*self.obj(),
                    //             "Error",
                    //             &format!("Failed to get visible surface: {}", e),
                    //         );
                    //         return;
                    //     }
                    // };
                } else {
                    w.file_view
                        .navigate_item(Direction::Down, &Filter::Liked, 1);
                }
            }
            Key::S => {
                if modifiers.contains(ModifierType::CONTROL_MASK) {
                    // Ctrl+Shift+S: save visible area on screen to a file
                    self.save_visible_area_to_file();
                    // let _surface = match w.image_view.imp().visible_to_surface() {
                    //     Ok(surface) => surface,
                    //     Err(e) => {
                    //         error_dialog(
                    //             &*self.obj(),
                    //             "Error",
                    //             &format!("Failed to get visible surface: {}", e),
                    //         );
                    //         return;
                    //     }
                    // };
                }
            }
            Key::Home => {
                self.reload(&Target::First, modifiers.contains(ModifierType::SHIFT_MASK));
            }
            Key::End => {
                self.reload(&Target::Last, modifiers.contains(ModifierType::SHIFT_MASK));
            }
            Key::F2 => {
                self.measure_toggle();
            }
            Key::Tab => {
                self.measure_move_endpoints();
            }
            Key::F6 => {
                contrast_delta(-1);
                dbg!(contrast());
            }
            Key::F7 => {
                contrast_delta(1);
                dbg!(contrast());
            }
            #[cfg(feature = "mupdf")]
            Key::F8 => {
                self.toggle_pdf_engine();
            }
            Key::_1 => {
                self.change_sort(Column::FileType, &w.file_view);
            }
            Key::_2 => {
                self.change_sort(Column::Name, &w.file_view);
            }
            Key::_3 => {
                self.change_sort(Column::Size, &w.file_view);
            }
            Key::_4 => {
                self.change_sort(Column::Modified, &w.file_view);
            }
            Key::p => {
                match self.page_mode.get() {
                    PageMode::DualEvenOdd => self.change_page_mode(PageMode::Single.into()),
                    PageMode::Single => self.change_page_mode(PageMode::DualOddEven.into()),
                    PageMode::DualOddEven => self.change_page_mode(PageMode::DualEvenOdd.into()),
                };
            }
            Key::P => {
                if modifiers.contains(ModifierType::CONTROL_MASK)
                    && modifiers.contains(ModifierType::SHIFT_MASK)
                {
                    let palette =
                        CommandPalette::new(&self.obj().clone(), self.recent_commands.clone());
                    palette.show();
                } else {
                    let w = self.widgets();
                    let params = ImageParams {
                        tn_sender: Some(&w.tn_sender),
                        page_mode: &self.page_mode.get(),
                        allocation_height: self.obj().height(),
                    };
                    if let Some((file_row, _)) = w.file_view.selected() {
                        let b = self.backend.borrow();
                        let image1 = b.content(&b.reference(&file_row).item, &params);
                        if let Some((file_row, _)) =
                            w.file_view.try_navigate(Direction::Down, &Filter::None, 1)
                        {
                            let image2 = b.content(&b.reference(&file_row).item, &params);
                            if let (ContentData::Single(single1), ContentData::Single(single2)) =
                                (image1.data, image2.data)
                            {
                                let i2 = Content::new_dual_surface(
                                    Some(single1.take_surface()),
                                    Some(single2.take_surface()),
                                    None,
                                );
                                w.info_view.update(&i2);
                                w.image_view.set_content(i2);
                            }
                        }
                    };
                }
            }
            Key::g => {
                let surface = match w.image_view.imp().visible_to_surface() {
                    Ok(surface) => surface,
                    Err(e) => {
                        error_dialog(
                            &*self.obj(),
                            "Error",
                            &format!("Failed to get visible surface: {}", e),
                        );
                        return;
                    }
                };
                let mut file = match File::create("/tmp/surface.png") {
                    Ok(file) => file,
                    Err(e) => {
                        error_dialog(
                            &*self.obj(),
                            "Error",
                            &format!("Failed to create file: {}", e),
                        );
                        return;
                    }
                };
                if let Err(e) = surface.write_to_png(&mut file) {
                    error_dialog(
                        &*self.obj(),
                        "Error",
                        &format!("Failed to write PNG: {}", e),
                    );
                }
            }
            _ => (),
        }
    }
}
