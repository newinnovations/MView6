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

use std::collections::HashSet;

use glib::{clone, subclass::types::ObjectSubclassExt, Propagation};
use gtk4::{
    gdk::Key, prelude::*, Box, Button, CheckButton, Dialog, EventControllerKey, Orientation,
    ResponseType, Separator,
};

use crate::{
    category::{ContentType, Preference},
    file_view::Filter,
    window::imp::MViewWindowImp,
};

const C_ITEMS: &[(&str, ContentType, Key)] = &[
    ("Images [i]", ContentType::Image, Key::i),
    ("Videos [v]", ContentType::Video, Key::v),
    ("Documents [d]", ContentType::Document, Key::d),
    ("Folders [f]", ContentType::Folder, Key::f),
    ("Archives [a]", ContentType::Archive, Key::a),
    ("Unsupported content [u]", ContentType::Unsupported, Key::u),
];

const F_ITEMS: &[(&str, Preference, Key)] = &[
    ("Normal items [n]", Preference::Normal, Key::n),
    ("Liked items [l]", Preference::Liked, Key::l),
    ("Disliked items [t]", Preference::Disliked, Key::t),
];

const A_ITEMS: &[(ContentType, Key)] = &[
    (ContentType::Image, Key::I),
    (ContentType::Video, Key::V),
    (ContentType::Document, Key::D),
    (ContentType::Archive, Key::A),
];

impl MViewWindowImp {
    pub fn filter_dialog(&self) {
        let dialog = Dialog::builder()
            .title("Navigation filter")
            .modal(true)
            .transient_for(&self.obj().clone())
            .build();

        let content_area = dialog.content_area();

        let hbox = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8) // vertical spacing between rows
            .margin_start(12)
            .margin_end(12)
            .margin_top(12)
            .margin_bottom(18)
            .build();

        let vbox_checks = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_start(12)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        let mut c_checks = Vec::new();
        let mut f_checks = Vec::new();
        if let Filter::Set((c_filter, f_filter)) = &*self.current_filter.borrow() {
            for (item, content_type, _) in C_ITEMS {
                let checkbox = CheckButton::with_label(item);
                checkbox.set_active(c_filter.contains(content_type));
                if let Some(label) = checkbox.last_child() {
                    label.set_margin_start(8)
                }
                vbox_checks.append(&checkbox);
                c_checks.push((checkbox, *content_type));
            }
            let separator = Separator::new(Orientation::Horizontal);
            separator.add_css_class("navsep");
            vbox_checks.append(&separator);
            for (item, pref_type, _) in F_ITEMS {
                let checkbox = CheckButton::with_label(item);
                checkbox.set_active(f_filter.contains(pref_type));
                if let Some(label) = checkbox.last_child() {
                    label.set_margin_start(8)
                }
                vbox_checks.append(&checkbox);
                f_checks.push((checkbox, *pref_type));
            }
        }

        let vbox_buttons = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(28)
            .margin_end(12)
            .margin_top(6)
            .build();

        let all_button = Button::with_label("Everything [E]");
        let cb_clone = c_checks.clone();
        let fb_clone = f_checks.clone();
        all_button.connect_clicked(move |_| {
            for (cb, _) in &cb_clone {
                cb.set_active(true);
            }
            for (cb, _) in &fb_clone {
                cb.set_active(true);
            }
        });

        let images_button = Button::with_label("Only images [I]");
        let cb_clone = c_checks.clone();
        let fb_clone = f_checks.clone();
        images_button.connect_clicked(move |_| {
            for (cb, ct) in &cb_clone {
                cb.set_active(*ct == ContentType::Image);
            }
            for (cb, preference) in &fb_clone {
                cb.set_active(*preference != Preference::Disliked);
            }
        });

        let videos_button = Button::with_label("Only videos [V]");
        let cb_clone = c_checks.clone();
        let fb_clone = f_checks.clone();
        videos_button.connect_clicked(move |_| {
            for (cb, ct) in &cb_clone {
                cb.set_active(*ct == ContentType::Video);
            }
            for (cb, preference) in &fb_clone {
                cb.set_active(*preference != Preference::Disliked);
            }
        });

        let archives_button = Button::with_label("Only archives [A]");
        let cb_clone = c_checks.clone();
        let fb_clone = f_checks.clone();
        archives_button.connect_clicked(move |_| {
            for (cb, ct) in &cb_clone {
                cb.set_active(*ct == ContentType::Archive);
            }
            for (cb, preference) in &fb_clone {
                cb.set_active(*preference != Preference::Disliked);
            }
        });

        let documents_button = Button::with_label("Only documents [D]");
        let cb_clone = c_checks.clone();
        let fb_clone = f_checks.clone();
        documents_button.connect_clicked(move |_| {
            for (cb, ct) in &cb_clone {
                cb.set_active(*ct == ContentType::Document);
            }
            for (cb, preference) in &fb_clone {
                cb.set_active(*preference != Preference::Disliked);
            }
        });

        vbox_buttons.append(&all_button);
        vbox_buttons.append(&images_button);
        vbox_buttons.append(&videos_button);
        vbox_buttons.append(&documents_button);
        vbox_buttons.append(&archives_button);

        let separator = Separator::new(Orientation::Vertical);
        separator.add_css_class("navsep");
        hbox.append(&vbox_buttons);
        hbox.append(&separator);
        hbox.append(&vbox_checks);
        content_area.append(&hbox);

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

        let cb_clone = c_checks.clone();
        let fb_clone = f_checks.clone();
        let key_controller = EventControllerKey::new();
        {
            let dialog_clone = dialog.clone();
            key_controller.connect_key_pressed(move |_, keyval, _, _| {
                for (_, content_type, key) in C_ITEMS {
                    if *key == keyval {
                        for (cb, cb_content) in &cb_clone {
                            if *content_type == *cb_content {
                                cb.set_active(!cb.is_active());
                                return Propagation::Stop;
                            }
                        }
                    }
                }
                for (_, preference, key) in F_ITEMS {
                    if *key == keyval {
                        for (cb, cb_preference) in &fb_clone {
                            if *preference == *cb_preference {
                                cb.set_active(!cb.is_active());
                                return Propagation::Stop;
                            }
                        }
                    }
                }
                for (content_type, key) in A_ITEMS {
                    if *key == keyval {
                        for (cb, ct) in &cb_clone {
                            cb.set_active(*ct == *content_type);
                        }
                        for (cb, preference) in &fb_clone {
                            cb.set_active(*preference != Preference::Disliked);
                        }
                    }
                }
                match keyval {
                    Key::E => {
                        for (cb, _) in &cb_clone {
                            cb.set_active(true);
                        }
                        for (cb, _) in &fb_clone {
                            cb.set_active(true);
                        }
                        Propagation::Stop
                    }
                    Key::Escape | Key::q | Key::Q => {
                        dialog_clone.response(ResponseType::Cancel);
                        Propagation::Stop
                    }
                    _ => Propagation::Proceed,
                }
            });
        }

        dialog.add_controller(key_controller);

        dialog.connect_response(clone!(
            #[weak(rename_to = this)]
            self,
            move |dialog, response| {
                if response == ResponseType::Ok {
                    let c_selected: HashSet<ContentType> = c_checks
                        .iter()
                        .filter(|&(cb, _)| cb.is_active())
                        .map(|(_, content_type)| *content_type)
                        .collect();
                    let f_selected: HashSet<Preference> = f_checks
                        .iter()
                        .filter(|&(cb, _)| cb.is_active())
                        .map(|(_, preference_type)| *preference_type)
                        .collect();
                    this.current_filter
                        .replace(Filter::Set((c_selected, f_selected)));
                }
                dialog.close();
            }
        ));

        dialog.present();
    }
}
