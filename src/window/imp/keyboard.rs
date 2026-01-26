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

use super::MViewWindowImp;

use glib::subclass::types::ObjectSubclassExt;
use gtk4::{
    gdk::{Key, ModifierType},
    prelude::{GtkWindowExt, WidgetExt},
};

use crate::{
    backends::{document::PageMode, Backend, ImageParams},
    config::{contrast, contrast_delta},
    content::{Content, ContentData},
    file_view::{Column, Direction, Filter, Target},
    image::view::ZoomMode,
    window::imp::palette::CommandPalette,
};

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, key: Key, modifiers: ModifierType) {
        let w = self.widgets();
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
                    let target = if let Some(cursor) = w.file_view.current() {
                        backend.reference(&cursor).into()
                    } else {
                        Target::First
                    };
                    self.set_backend(<dyn Backend>::bookmarks(backend, target), &Target::First);
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
                self.filter_dialog();
            }
            Key::Escape => {
                self.obj().unfullscreen();
                self.fullscreen.set(false);
                self.widgets().set_action_bool("fullscreen", false);
                w.image_view.measure_enable(false);
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
                w.file_view.set_unsorted();
                if let Some(current) = w.file_view.current() {
                    if self
                        .backend
                        .borrow()
                        .set_preference(&current, Direction::Down)
                    {
                        w.file_view
                            .navigate_item(Direction::Down, &Filter::Image, 1);
                    }
                }
            }
            Key::equal | Key::KP_Add => {
                w.file_view.set_unsorted();
                if let Some(current) = w.file_view.current() {
                    if self
                        .backend
                        .borrow()
                        .set_preference(&current, Direction::Up)
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
                w.file_view
                    .navigate_item(Direction::Down, &Filter::Liked, 1);
            }
            Key::Up | Key::z => {
                w.file_view.navigate_item(
                    Direction::Up,
                    &self.current_filter.borrow(),
                    self.step_size(),
                );
            }
            Key::Down | Key::x => {
                w.file_view.navigate_item(
                    Direction::Down,
                    &self.current_filter.borrow(),
                    self.step_size(),
                );
            }
            Key::Z | Key::Left | Key::KP_4 | Key::KP_Left => {
                self.navigate_page(Direction::Up, self.step_size());
            }
            Key::X | Key::Right | Key::KP_6 | Key::KP_Right => {
                self.navigate_page(Direction::Down, self.step_size());
            }
            Key::KP_8 | Key::KP_Up => {
                w.file_view
                    .navigate_item(Direction::Up, &self.current_filter.borrow(), 5);
            }
            Key::KP_2 | Key::KP_Down => {
                w.file_view
                    .navigate_item(Direction::Down, &self.current_filter.borrow(), 5);
            }
            Key::Page_Up => {
                w.file_view
                    .navigate_item(Direction::Up, &self.current_filter.borrow(), 25);
            }
            Key::Page_Down => {
                w.file_view
                    .navigate_item(Direction::Down, &self.current_filter.borrow(), 25);
            }
            Key::Home => {
                self.reload(&Target::First);
            }
            Key::End => {
                self.reload(&Target::Last);
            }
            Key::F2 => {
                self.measure_toggle();
            }
            Key::Tab => {
                self.measure_move_endpoints();
                // // set reference
                // let mouse = w.image_view.mouse_position();
                // let img = w.image_view.zoom().screen_to_image(&mouse);
                // w.image_view.measure_anchor(img);
                // // self.measurement_reference.replace(img);
            }
            // Key::F3 => {
            //     // measure
            //     let mouse = w.image_view.mouse_position();
            //     let img = w.image_view.zoom().screen_to_image(&mouse);
            //     if let Some(text) = w.image_view.measure_point(img) {
            //         self.copy_to_clipboard(&text);
            //     };
            //     // let reference = self.measurement_reference.get();
            //     // let delta = img - reference;
            //     // let distance = img.distance(&reference);
            //     // let factor = 2.54 / 600.0; // 600 dpi
            //     // println!(
            //     //     "dx {:8.3}   dy {:8.3}   dist {:8.3}",
            //     //     delta.x() * factor,
            //     //     delta.y() * factor,
            //     //     distance * factor
            //     // );
            //     // dbg!(img, reference, delta);
            // }
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
                    if let Some(current) = w.file_view.current() {
                        let b = self.backend.borrow();
                        let image1 = b.content(&b.reference(&current).item, &params);
                        if current.next() {
                            let image2 = b.content(&b.reference(&current).item, &params);
                            if let (ContentData::Single(single1), ContentData::Single(single2)) =
                                (image1.data, image2.data)
                            {
                                let i2 = Content::new_dual_surface(
                                    Some(single1.surface()),
                                    Some(single2.surface()),
                                    None,
                                );
                                w.info_view.update(&i2);
                                w.image_view.set_content(i2);
                            }
                        }
                    };
                }
            }
            _ => (),
        }
    }
}
