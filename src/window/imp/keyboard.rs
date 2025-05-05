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

use super::MViewWindowImp;

use gtk4::{gdk::Key, prelude::*, subclass::prelude::*, SortColumn};

use crate::{
    backends::{document::PageMode, thumbnail::Thumbnail, Backend},
    file_view::{Direction, Filter, Selection, Sort},
    image::{provider::ImageLoader, view::ZoomMode, Image, ImageData},
};

impl MViewWindowImp {
    pub(super) fn on_key_press(&self, e: Key) {
        let w = self.widgets();
        match e {
            Key::q => {
                self.obj().close();
            }
            Key::h => {
                let image = if w.image_view.has_tag("help1") {
                    ImageLoader::image_from_svg_data(
                        include_bytes!("../../../resources/mv6-help-2.svgz"),
                        Some("help2".to_string()),
                    )
                } else {
                    ImageLoader::image_from_svg_data(
                        include_bytes!("../../../resources/mv6-help-1.svgz"),
                        Some("help1".to_string()),
                    )
                };
                if let Some(image) = image {
                    w.image_view.set_image(image);
                }
            }
            Key::y => {
                let image =
                    ImageLoader::image_from_file("/home/martin/workspace/rust/test/test-19.jpg");
                let image2 =
                    ImageLoader::image_from_file("/home/martin/workspace/rust/test/test-20.jpg");
                if let (ImageData::Single(pixbuf), ImageData::Single(pixbuf2)) =
                    (image.image_data, image2.image_data)
                {
                    let i2 = Image::new_dual_pixbuf(Some(pixbuf), Some(pixbuf2), None);
                    w.info_view.update(&i2);
                    w.image_view.set_image(i2);
                }
            }
            Key::d => {
                self.show_files_widget(true);
                if !self.backend.borrow().is_bookmarks() {
                    self.set_backend(<dyn Backend>::bookmarks(), Selection::None, true);
                }
            }
            Key::i => {
                if !self.backend.borrow().is_thumbnail() {
                    self.show_info_widget(!w.info_widget.is_visible());
                }
            }
            Key::t => {
                if self.backend.borrow().is_container() {
                    let position = if let Some(cursor) = w.file_view.current() {
                        cursor.position()
                    } else {
                        0
                    };
                    if let Some(thumbnail) = Thumbnail::new(
                        w.image_view.allocation(),
                        position,
                        self.thumbnail_size.get(),
                    ) {
                        let startpage = thumbnail.startpage();
                        let new_backend = <dyn Backend>::thumbnail(thumbnail);
                        new_backend.set_sort(&Sort::sort_on_category());
                        self.set_backend(new_backend, startpage, true);
                        self.show_info_widget(false);
                        w.file_view.set_extended(false);
                    }
                }
            }
            Key::w | Key::KP_7 | Key::KP_Home => {
                self.hop(Direction::Up);
            }
            Key::e | Key::KP_9 | Key::KP_Page_Up => {
                self.hop(Direction::Down);
            }
            Key::space | Key::KP_Divide => {
                self.show_files_widget(!w.file_widget.is_visible());
            }
            Key::f | Key::KP_Multiply => {
                if self.full_screen.get() {
                    self.obj().unfullscreen();
                    self.full_screen.set(false);
                } else {
                    self.show_files_widget(false);
                    self.obj().fullscreen();
                    self.full_screen.set(true);
                }
                // w.image_view.update_mouse_position();
            }
            Key::Escape => {
                self.obj().unfullscreen();
                self.full_screen.set(false);
                // w.image_view.update_mouse_position();
            }
            Key::r => {
                w.image_view.rotate(270);
            }
            Key::R => {
                w.image_view.rotate(90);
            }
            Key::Return | Key::KP_Enter => {
                self.dir_enter(None);
            }
            Key::BackSpace | Key::KP_Delete | Key::KP_Decimal => {
                self.dir_leave();
                if self.backend.borrow().is_thumbnail() {
                    self.show_files_widget(false);
                }
            }
            Key::n => {
                if w.image_view.zoom_mode() == ZoomMode::Fit {
                    w.image_view.set_zoom_mode(ZoomMode::NoZoom);
                } else {
                    w.image_view.set_zoom_mode(ZoomMode::Fit);
                }
            }
            Key::m | Key::KP_0 | Key::KP_Insert => {
                let backend = self.backend.borrow();
                if backend.is_thumbnail() {
                    let new_size = match self.thumbnail_size.get() {
                        80 => 100,
                        100 => 140,
                        175 => 250,
                        250 => 80,
                        _ => 175,
                    };
                    self.thumbnail_size.set(new_size);
                    drop(backend);
                    self.update_thumbnail_backend()
                } else if w.image_view.zoom_mode() == ZoomMode::Max {
                    w.image_view.set_zoom_mode(ZoomMode::Fill);
                } else {
                    w.image_view.set_zoom_mode(ZoomMode::Max);
                }
            }
            Key::minus | Key::KP_Subtract => {
                w.file_view.set_unsorted();
                if let Some(current) = w.file_view.current() {
                    if self.backend.borrow().favorite(&current, Direction::Down) {
                        w.file_view.navigate(Direction::Down, Filter::Image, 1);
                    }
                }
            }
            Key::equal | Key::KP_Add => {
                w.file_view.set_unsorted();
                if let Some(current) = w.file_view.current() {
                    if self.backend.borrow().favorite(&current, Direction::Up) {
                        w.file_view.navigate(Direction::Down, Filter::Image, 1);
                    }
                }
            }
            Key::z | Key::Left | Key::KP_4 | Key::KP_Left => {
                w.file_view
                    .navigate(Direction::Up, w.filter(), self.step_size());
            }
            Key::x | Key::Right | Key::KP_6 | Key::KP_Right => {
                w.file_view
                    .navigate(Direction::Down, w.filter(), self.step_size());
            }
            Key::a => {
                w.file_view.navigate(Direction::Up, Filter::Favorite, 1);
            }
            Key::s => {
                w.file_view.navigate(Direction::Down, Filter::Favorite, 1);
            }
            Key::Z => {
                w.file_view.navigate(Direction::Up, Filter::None, 1);
            }
            Key::X => {
                w.file_view.navigate(Direction::Down, Filter::None, 1);
            }
            Key::Up | Key::KP_8 | Key::KP_Up => {
                w.file_view.navigate(Direction::Up, w.filter(), 5);
            }
            Key::Down | Key::KP_2 | Key::KP_Down => {
                w.file_view.navigate(Direction::Down, w.filter(), 5);
            }
            Key::Page_Up => {
                w.file_view.navigate(Direction::Up, w.filter(), 25);
            }
            Key::Page_Down => {
                w.file_view.navigate(Direction::Down, w.filter(), 25);
            }
            Key::Home => {
                w.file_view.home();
            }
            Key::End => {
                w.file_view.end();
            }
            Key::_1 => {
                if let Some(current) = w.file_view.current() {
                    current.set_sort_column(SortColumn::Index(0));
                    w.file_view.goto(&Selection::None);
                }
            }
            Key::_2 => {
                if !self.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_view.current() {
                        current.set_sort_column(SortColumn::Index(2));
                        w.file_view.goto(&Selection::None);
                    }
                }
            }
            Key::_3 => {
                if !self.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_view.current() {
                        current.set_sort_column(SortColumn::Index(3));
                        w.file_view.goto(&Selection::None);
                    }
                }
            }
            Key::_4 => {
                if !self.backend.borrow().is_thumbnail() {
                    if let Some(current) = w.file_view.current() {
                        current.set_sort_column(SortColumn::Index(4));
                        w.file_view.goto(&Selection::None);
                    }
                }
            }
            Key::p => {
                let new_page_mode = match self.page_mode.get() {
                    PageMode::Single => PageMode::DualOdd,
                    PageMode::DualOdd => PageMode::DualEven,
                    PageMode::DualEven => PageMode::Single,
                };
                dbg!(new_page_mode);
                self.page_mode.set(new_page_mode);
                if self.backend.borrow().is_doc() {
                    self.on_cursor_changed();
                }
            }
            _ => (),
        }
    }
}
