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

use super::info_view_imp::InfoRow;
use crate::content::Content;
use convert_case::{Case, Casing};
use exif::In;
use glib::subclass::types::ObjectSubclassIsExt;
use gtk4::{gio, glib};

glib::wrapper! {
pub struct InfoView(ObjectSubclass<super::InfoViewImp>)
    @extends gtk4::Box, gtk4::Widget,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl InfoView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn column_view(&self) -> &gtk4::ColumnView {
        self.imp().column_view.get().unwrap()
    }
}

impl InfoView {
    pub fn update(&self, image: &Content) {
        let store = gio::ListStore::new::<InfoRow>();

        let size = image.size();
        store.append(&InfoRow::new("width", &format!("{:.0} px", size.width())));
        store.append(&InfoRow::new("height", &format!("{:.0} px", size.height())));
        store.append(&InfoRow::new(
            "alpha channel",
            if image.has_alpha() { "yes" } else { "no" },
        ));

        // Rotation and mirror state live in the ImageView/Zoom, not in Content, so they
        // start out as placeholders here and are set to their real values right after by
        // update_transform(). We keep a reference to these two rows so that later changes
        // to rotation/mirror (e.g. from keyboard shortcuts) can update them in place via
        // InfoRow's properties, instead of rebuilding this whole ListStore.
        let rotation_row = InfoRow::new("rotation", "0°");
        let mirror_row = InfoRow::new("mirrored", "no");
        store.append(&rotation_row);
        store.append(&mirror_row);
        self.imp().rotation_row.replace(Some(rotation_row));
        self.imp().mirror_row.replace(Some(mirror_row));

        match &image.exif {
            Some(exif) => {
                for f in exif.fields() {
                    if f.ifd_num == In::PRIMARY {
                        let key = f.tag.to_string();
                        let key = key.from_case(Case::Pascal).to_case(Case::Lower);
                        // println!("{}", key);
                        if key != "maker note" && !key.starts_with("tag(") {
                            store.append(&InfoRow::new(
                                &key,
                                &f.display_value().with_unit(exif).to_string(),
                            ));
                        }
                    }
                }
            }
            None => {
                // println!("No exif data");
            }
        }
        let selection_model = gtk4::SingleSelection::new(Some(store));
        self.column_view().set_model(Some(&selection_model));
    }

    /// Update the rotation/mirror rows in place. This is cheap: it only sets two
    /// properties on the existing InfoRow objects (which the ColumnView labels are bound
    /// to via property expressions), so it avoids rebuilding the whole ListStore every
    /// time the user rotates or mirrors the image.
    pub fn update_transform(&self, rotation: i32, mirrored: bool) {
        if let Some(row) = self.imp().rotation_row.borrow().as_ref() {
            match rotation {
                0 => row.set_value("none"),
                90 => row.set_value("90° clockwise"),
                270 => row.set_value("90° counterclockwise"),
                _ => row.set_value(format!("{}°", rotation).as_str()),
            }
        }
        if let Some(row) = self.imp().mirror_row.borrow().as_ref() {
            row.set_value(if mirrored { "yes" } else { "no" });
        }
    }
}
