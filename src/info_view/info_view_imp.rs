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

use std::cell::{OnceCell, RefCell};

use glib::subclass::{
    object::{ObjectImpl, ObjectImplExt},
    types::{ObjectSubclass, ObjectSubclassExt},
};
use gtk4::{
    glib,
    prelude::*,
    subclass::{prelude::BoxImpl, widget::WidgetImpl},
    Box as GtkBox, ColumnView, ColumnViewColumn,
};

pub mod info_row {
    use gtk4::glib;

    glib::wrapper! {
        pub struct InfoRow(ObjectSubclass<imp::InfoRow>);
    }

    impl InfoRow {
        pub fn new(key: &str, value: &str) -> Self {
            glib::Object::builder()
                .property("key", key)
                .property("value", value)
                .build()
        }
    }

    mod imp {
        use gtk4::glib;
        use gtk4::prelude::ObjectExt;
        use gtk4::subclass::prelude::*;
        use std::cell::RefCell;

        #[derive(Default, glib::Properties)]
        #[properties(wrapper_type = super::InfoRow)]
        pub struct InfoRow {
            #[property(get, set)]
            pub key: RefCell<String>,
            #[property(get, set)]
            pub value: RefCell<String>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for InfoRow {
            const NAME: &'static str = "InfoRow";
            type Type = super::InfoRow;
            type ParentType = glib::Object;
        }

        #[glib::derived_properties]
        impl ObjectImpl for InfoRow {}
    }
}
pub use info_row::InfoRow;

use super::InfoView;

#[derive(Debug, Default)]
pub struct InfoViewImp {
    pub(super) column_view: OnceCell<ColumnView>,
    // Kept around so rotation/mirror can be refreshed in place (see InfoView::update_transform)
    // instead of rebuilding the whole ListStore whenever the rotation or mirror state changes.
    pub(super) rotation_row: RefCell<Option<InfoRow>>,
    pub(super) mirror_row: RefCell<Option<InfoRow>>,
}

#[glib::object_subclass]
impl ObjectSubclass for InfoViewImp {
    const NAME: &'static str = "InfoView";
    type Type = InfoView;
    type ParentType = GtkBox;
}

const WIDTH_KEY: i32 = 110;
const WIDTH_VALUE: i32 = 210;
const PADDING_X: i32 = 2;
const PADDING_Y: i32 = 3;

impl ObjectImpl for InfoViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let instance = self.obj();
        instance.set_halign(gtk4::Align::Start);

        let column_view = ColumnView::new(None::<gtk4::SingleSelection>);
        column_view.add_css_class("info-view");
        column_view.set_vexpand(true);
        column_view.set_hexpand(false);
        column_view.set_halign(gtk4::Align::Start);

        let factory_key = gtk4::SignalListItemFactory::new();
        factory_key.connect_setup(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = gtk4::Label::builder()
                .halign(gtk4::Align::Start)
                .valign(gtk4::Align::Start)
                .wrap(true)
                .wrap_mode(gtk4::pango::WrapMode::WordChar)
                .width_request(WIDTH_KEY)
                .xalign(0.0)
                .build();
            label.set_margin_start(PADDING_X);
            label.set_margin_end(PADDING_X);
            label.set_margin_top(PADDING_Y);
            label.set_margin_bottom(PADDING_Y);
            list_item.set_child(Some(&label));
        });
        factory_key.connect_bind(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = list_item
                .child()
                .and_then(|w| w.downcast::<gtk4::Label>().ok())
                .unwrap();
            // Bind the label text to the InfoRow "key" property via the list item's
            // "item" property. As the expression watches list_item::item, it
            // automatically re-binds when the item is recycled, and it also picks up
            // property changes on the InfoRow itself (e.g. when we update rotation or
            // mirror in place), without needing to rebuild the whole ListStore.
            list_item
                .property_expression("item")
                .chain_property::<InfoRow>("key")
                .bind(&label, "label", gtk4::Widget::NONE);
        });

        let col_key = ColumnViewColumn::new(Some("Key"), Some(factory_key.clone()));
        col_key.set_fixed_width(WIDTH_KEY + 4 * PADDING_X);
        column_view.append_column(&col_key);

        let factory_value = gtk4::SignalListItemFactory::new();
        factory_value.connect_setup(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = gtk4::Label::builder()
                .halign(gtk4::Align::Start)
                .valign(gtk4::Align::Start)
                .wrap(true)
                .wrap_mode(gtk4::pango::WrapMode::WordChar)
                .width_request(WIDTH_VALUE)
                .xalign(0.0)
                .build();
            label.set_margin_start(PADDING_X);
            label.set_margin_end(PADDING_X);
            label.set_margin_top(PADDING_Y);
            label.set_margin_bottom(PADDING_Y);
            list_item.set_child(Some(&label));
        });
        factory_value.connect_bind(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            let label = list_item
                .child()
                .and_then(|w| w.downcast::<gtk4::Label>().ok())
                .unwrap();
            // See factory_key above: binding to the "value" property lets us update
            // e.g. the rotation/mirror rows in place (InfoRow::set_value) and have the
            // label refresh automatically, without rebuilding the ListStore.
            list_item
                .property_expression("item")
                .chain_property::<InfoRow>("value")
                .bind(&label, "label", gtk4::Widget::NONE);
        });

        let col_value = ColumnViewColumn::new(Some("Value"), Some(factory_value.clone()));
        col_value.set_fixed_width(WIDTH_VALUE + 4 * PADDING_X);
        column_view.append_column(&col_value);

        instance.append(&column_view);
        self.column_view.set(column_view).unwrap();
    }
}

impl WidgetImpl for InfoViewImp {}

impl BoxImpl for InfoViewImp {}

impl InfoViewImp {}
