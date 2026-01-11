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

use glib::{clone, subclass::types::ObjectSubclassExt, Propagation};
use gtk4::{
    gdk::Key, prelude::*, Box, CheckButton, Dialog, EventControllerKey, Orientation, ResponseType,
    Separator,
};

use crate::{
    category::Category,
    file_view::{Filter, FilterSet},
    window::imp::MViewWindowImp,
};

const ITEMS: &[(&str, Category)] = &[
    ("Images", Category::Image),
    ("Documents", Category::Document),
    ("Folders", Category::Folder),
    ("Archives", Category::Archive),
    ("Unsupported content", Category::Unsupported),
    ("Favorite items", Category::Favorite),
    ("Trashed items", Category::Trash),
];

impl MViewWindowImp {
    pub fn filter_dialog(&self) {
        let dialog = Dialog::builder()
            .title("Navigation filter")
            .modal(true)
            .transient_for(&self.obj().clone())
            .build();

        let content_area = dialog.content_area();
        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8) // vertical spacing between rows
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(12)
            .build();

        let mut checkboxes = Vec::new();
        if let Filter::Set(filter) = &*self.current_filter.borrow() {
            for (item, category) in ITEMS {
                if *category == Category::Favorite {
                    vbox.append(&Separator::new(Orientation::Horizontal));
                }
                let checkbox = CheckButton::with_label(item);
                checkbox.set_active(filter.contains(category));
                if let Some(label) = checkbox.last_child() {
                    label.set_margin_start(8)
                }
                vbox.append(&checkbox);
                checkboxes.push((checkbox, *category));
            }
        }

        content_area.append(&vbox);

        let cancel_btn = dialog.add_button("Cancel", ResponseType::Cancel);
        cancel_btn.set_margin_end(8); // space to the right of Cancel
        cancel_btn.set_margin_bottom(8);

        let ok_btn = dialog.add_button("OK", ResponseType::Ok);
        ok_btn.set_margin_start(8);
        ok_btn.set_margin_end(8);
        ok_btn.set_margin_bottom(8);

        // Prevent focus outline on the first checkbox by focusing OK when shown
        let ok_btn_clone = ok_btn.clone();
        dialog.connect_show(move |_| {
            ok_btn_clone.grab_focus();
        });

        let key_controller = EventControllerKey::new();
        {
            let dialog_clone = dialog.clone();
            key_controller.connect_key_pressed(move |_, keyval, _, _| match keyval {
                Key::Escape | Key::q | Key::Q => {
                    dialog_clone.response(ResponseType::Cancel);
                    Propagation::Stop
                }
                _ => Propagation::Proceed,
            });
        }

        dialog.add_controller(key_controller);

        dialog.connect_response(clone!(
            #[weak(rename_to = this)]
            self,
            move |dialog, response| {
                if response == ResponseType::Ok {
                    let selected: FilterSet = checkboxes
                        .iter()
                        .filter(|&(cb, _)| cb.is_active())
                        .map(|(_, category)| *category)
                        .collect();
                    this.current_filter.replace(Filter::Set(selected));
                }
                dialog.close();
            }
        ));

        dialog.present();
    }
}
