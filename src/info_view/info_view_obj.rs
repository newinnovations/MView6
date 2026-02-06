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

use super::Columns;
use crate::content::Content;
use convert_case::{Case, Casing};
use exif::In;
use gtk4::{glib, prelude::TreeViewExt};

glib::wrapper! {
pub struct InfoView(ObjectSubclass<super::InfoViewImp>)
    @extends gtk4::Widget, gtk4::TreeView,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Scrollable;
}

impl InfoView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl InfoView {
    pub fn update(&self, image: &Content) {
        let store = Columns::store();

        let size = image.size();
        Columns::insert(&store, "width", &format!("{:.0} px", size.width()));
        Columns::insert(&store, "height", &format!("{:.0} px", size.height()));
        Columns::insert(
            &store,
            "alpha channel",
            if image.has_alpha() { "yes" } else { "no" },
        );

        match &image.exif {
            Some(exif) => {
                for f in exif.fields() {
                    if f.ifd_num == In::PRIMARY {
                        let key = f.tag.to_string();
                        let key = key.from_case(Case::Pascal).to_case(Case::Lower);
                        // println!("{}", key);
                        if key != "maker note" && !key.starts_with("tag(") {
                            Columns::insert(
                                &store,
                                &key,
                                &f.display_value().with_unit(exif).to_string(),
                            )
                        }
                    }
                }
            }
            None => {
                // println!("No exif data");
            }
        }
        self.set_model(Some(&store));
    }
}
