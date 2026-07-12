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

use crate::file_view;
use chrono::{
    offset::LocalResult,
    {Local, TimeZone},
};
use glib::subclass::{
    object::{ObjectImpl, ObjectImplExt},
    types::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt},
};
use gtk4::{
    glib,
    prelude::*,
    subclass::{prelude::BoxImpl, widget::WidgetImpl},
    Box as GtkBox, ColumnView, ColumnViewColumn,
};
use human_bytes::human_bytes;

use super::FileRow;
#[derive(Debug)]
#[allow(dead_code)]
pub(super) struct FileViewColumns {
    pub(super) category: ColumnViewColumn,
    pub(super) name: ColumnViewColumn,
    pub(super) size: ColumnViewColumn,
    pub(super) date: ColumnViewColumn,
}

#[derive(Debug)]
pub(super) struct FileViewSorters {
    pub(super) category: gtk4::CustomSorter,
    pub(super) category_desc: gtk4::CustomSorter,
    pub(super) name: gtk4::CustomSorter,
    pub(super) name_desc: gtk4::CustomSorter,
    pub(super) size: gtk4::CustomSorter,
    pub(super) size_desc: gtk4::CustomSorter,
    pub(super) date: gtk4::CustomSorter,
    pub(super) date_desc: gtk4::CustomSorter,
    pub(super) index: gtk4::CustomSorter,
}

#[derive(Default)]
pub struct FileViewImp {
    pub(super) columns: OnceCell<FileViewColumns>,
    pub(super) selection_changed_callback: RefCell<Option<Box<dyn Fn() + 'static>>>,
    pub(super) sorters: OnceCell<FileViewSorters>,
    pub(super) sort_model_sorter: OnceCell<gtk4::MultiSorter>,
    pub(super) column_view: OnceCell<ColumnView>,
}

#[glib::object_subclass]
impl ObjectSubclass for FileViewImp {
    const NAME: &'static str = "FileListView";
    type Type = file_view::FileView;
    type ParentType = GtkBox;
}

impl FileViewImp {
    pub(super) fn set_extended(&self, extended: bool) {
        let columns = self.columns.get().unwrap();
        if extended != columns.size.is_visible() {
            columns.size.set_visible(extended);
            columns.date.set_visible(extended);
        }
    }
}

impl ObjectImpl for FileViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let instance = self.obj();
        instance.set_halign(gtk4::Align::Start);

        let column_view = ColumnView::new(None::<gtk4::SingleSelection>);
        column_view.add_css_class("file-view");
        column_view.set_vexpand(true);
        column_view.set_hexpand(false);
        column_view.set_halign(gtk4::Align::Start);

        // Set up custom sorters for each column
        let sorter_type = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<FileRow>().unwrap();
            let b = b.downcast_ref::<FileRow>().unwrap();
            // let a_row = a.row();
            // let b_row = b.row();
            // let a_val = a_row.as_ref().unwrap();
            // let b_val = b_row.as_ref().unwrap();
            a.file_type().cmp(&b.file_type()).into()
        });
        let sorter_type_desc = {
            let sorter_type = sorter_type.clone();
            gtk4::CustomSorter::new(move |a, b| sorter_type.compare(b, a))
        };

        let sorter_name = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<FileRow>().unwrap();
            let b = b.downcast_ref::<FileRow>().unwrap();
            // let a_row = a.row();
            // let b_row = b.row();
            // let a_val = a_row.as_ref().unwrap();
            // let b_val = b_row.as_ref().unwrap();
            a.name().to_lowercase().cmp(&b.name().to_lowercase()).into()
        });
        let sorter_name_desc = {
            let sorter_name = sorter_name.clone();
            gtk4::CustomSorter::new(move |a, b| sorter_name.compare(b, a))
        };

        let sorter_size = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<FileRow>().unwrap();
            let b = b.downcast_ref::<FileRow>().unwrap();
            // let a_row = a.row();
            // let b_row = b.row();
            // let a_val = a_row.as_ref().unwrap();
            // let b_val = b_row.as_ref().unwrap();
            a.size().cmp(&b.size()).into()
        });
        let sorter_size_desc = {
            let sorter_size = sorter_size.clone();
            gtk4::CustomSorter::new(move |a, b| sorter_size.compare(b, a))
        };

        let sorter_modified = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<FileRow>().unwrap();
            let b = b.downcast_ref::<FileRow>().unwrap();
            // let a_row = a.row();
            // let b_row = b.row();
            // let a_val = a_row.as_ref().unwrap();
            // let b_val = b_row.as_ref().unwrap();
            a.modified().cmp(&b.modified()).into()
        });
        let sorter_modified_desc = {
            let sorter_modified = sorter_modified.clone();
            gtk4::CustomSorter::new(move |a, b| sorter_modified.compare(b, a))
        };
        let sorter_index = gtk4::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<FileRow>().unwrap();
            let b = b.downcast_ref::<FileRow>().unwrap();
            // let a_row = a.row();
            // let b_row = b.row();
            // let a_val = a_row.as_ref().unwrap();
            // let b_val = b_row.as_ref().unwrap();
            a.index().cmp(&b.index()).into()
        });

        self.sorters
            .set(FileViewSorters {
                category: sorter_type.clone(),
                category_desc: sorter_type_desc,
                name: sorter_name.clone(),
                name_desc: sorter_name_desc,
                size: sorter_size.clone(),
                size_desc: sorter_size_desc,
                date: sorter_modified.clone(),
                date_desc: sorter_modified_desc,
                index: sorter_index,
            })
            .unwrap();
        self.sort_model_sorter
            .set(gtk4::MultiSorter::new())
            .expect("Failed to create file list multi sorter");

        // Column for category (FileType)
        let factory_category = gtk4::SignalListItemFactory::new();
        factory_category.connect_setup(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk4::ListItem>().unwrap();
            list_item.set_activatable(true);
            let image = gtk4::Image::builder()
                .icon_size(gtk4::IconSize::Normal)
                .build();
            list_item.set_child(Some(&image));
        });
        factory_category.connect_bind(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk4::ListItem>() else {
                return;
            };
            let Some(image) = list_item
                .child()
                .and_then(|w| w.downcast::<gtk4::Image>().ok())
            else {
                return;
            };
            let Some(file_row) = list_item
                .item()
                .and_then(|obj| obj.downcast::<FileRow>().ok())
            else {
                return;
            };
            // let row = file_row.row();
            // let Some(row) = row.as_ref() else {
            //     return;
            // };
            // image.set_icon_name(Some(&file_row.icon()));
            file_row
                .bind_property("file-icon", &image, "icon-name")
                .sync_create() // Applies the current initial state immediately
                .build();
        });
        let col_category = ColumnViewColumn::new(None, Some(factory_category.clone()));
        col_category.set_fixed_width(30);
        //col_category.set_sorter(Some(&sorter_type));

        column_view.append_column(&col_category);

        // Column for file/directory name
        let factory_name = gtk4::SignalListItemFactory::new();
        factory_name.connect_setup(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk4::ListItem>() else {
                return;
            };
            list_item.set_activatable(true);
            let box_widget = gtk4::Box::builder()
                .orientation(gtk4::Orientation::Horizontal)
                .spacing(4)
                .build();
            let image = gtk4::Image::builder()
                .icon_size(gtk4::IconSize::Normal)
                .build();
            let label = gtk4::Label::builder().halign(gtk4::Align::Start).build();
            box_widget.append(&image);
            box_widget.append(&label);
            list_item.set_child(Some(&box_widget));
        });
        factory_name.connect_bind(|_, list_item| {
            // 1. Safe downcast of the list item
            let Some(list_item) = list_item.downcast_ref::<gtk4::ListItem>() else {
                return;
            };

            // 2. Safe retrieval and downcast of the child Box
            let Some(box_widget) = list_item
                .child()
                .and_then(|w| w.downcast::<gtk4::Box>().ok())
            else {
                return;
            };

            // 3. Safe retrieval of the Image child
            let Some(image) = box_widget
                .first_child()
                .and_then(|w| w.downcast::<gtk4::Image>().ok())
            else {
                return;
            };

            // 4. Safe retrieval of the Label sibling
            let Some(label) = image
                .next_sibling()
                .and_then(|w| w.downcast::<gtk4::Label>().ok())
            else {
                return;
            };

            // 5. Safe retrieval of the underlying FileRow object
            let Some(file_row) = list_item
                .item()
                .and_then(|obj| obj.downcast::<FileRow>().ok())
            else {
                return;
            };

            // // 6. Safe reference check for the row data
            // let row = file_row.row();
            // let Some(row) = row.as_ref() else {
            //     return;
            // };

            // 7. Safe UI Updates
            // image.set_icon_name(Some(file_row.preference().icon()));
            // image.set_visible(file_row.preference().show_icon());

            file_row
                .bind_property("pref-icon", &image, "icon-name")
                .sync_create() // Applies the current initial state immediately
                .build();

            file_row
                .bind_property("pref-icon-visible", &image, "visible")
                .sync_create() // Applies the current initial state immediately
                .build();

            label.set_text(&file_row.name());
        });
        let col_name = ColumnViewColumn::new(Some("Name"), Some(factory_name.clone()));
        col_name.set_fixed_width(300);
        //col_name.set_sorter(Some(&sorter_name));
        column_view.append_column(&col_name);

        // Column for size
        let factory_size = gtk4::SignalListItemFactory::new();
        factory_size.connect_setup(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk4::ListItem>() else {
                return;
            };
            list_item.set_activatable(true);
            let label = gtk4::Label::builder().halign(gtk4::Align::End).build();
            list_item.set_child(Some(&label));
        });
        factory_size.connect_bind(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk4::ListItem>() else {
                return;
            };
            let Some(label) = list_item
                .child()
                .and_then(|w| w.downcast::<gtk4::Label>().ok())
            else {
                return;
            };
            let Some(file_row) = list_item
                .item()
                .and_then(|obj| obj.downcast::<FileRow>().ok())
            else {
                return;
            };
            // let row = file_row.row();
            // let Some(row) = row.as_ref() else {
            //     return;
            // };
            let size = file_row.size();
            let modified_text = if size > 0 {
                human_bytes(size as f64)
            } else {
                String::default()
            };
            label.set_text(&modified_text);
        });
        let col_size = ColumnViewColumn::new(Some("Size"), Some(factory_size.clone()));
        col_size.set_fixed_width(90);
        //col_size.set_sorter(Some(&sorter_size));
        column_view.append_column(&col_size);

        // Column for modified date
        let factory_date = gtk4::SignalListItemFactory::new();
        factory_date.connect_setup(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk4::ListItem>() else {
                return;
            };
            list_item.set_activatable(true);
            let label = gtk4::Label::builder().halign(gtk4::Align::Start).build();
            list_item.set_child(Some(&label));
        });
        factory_date.connect_bind(|_, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk4::ListItem>() else {
                return;
            };
            let Some(label) = list_item
                .child()
                .and_then(|w| w.downcast::<gtk4::Label>().ok())
            else {
                return;
            };
            let Some(file_row) = list_item
                .item()
                .and_then(|obj| obj.downcast::<FileRow>().ok())
            else {
                return;
            };
            let modified = file_row.modified();
            let modified_text = if modified > 0 {
                if let LocalResult::Single(dt) = Local.timestamp_opt(modified as i64, 0) {
                    dt.format("%d-%m-%Y %H:%M:%S").to_string()
                } else {
                    String::default()
                }
            } else {
                String::default()
            };
            label.set_text(&modified_text);
        });
        let col_date = ColumnViewColumn::new(Some("Modified"), Some(factory_date.clone()));
        col_date.set_fixed_width(if cfg!(target_os = "windows") {
            147
        } else {
            142
        });
        //col_date.set_sorter(Some(&sorter_modified));
        column_view.append_column(&col_date);

        self.columns
            .set(FileViewColumns {
                category: col_category,
                name: col_name,
                size: col_size,
                date: col_date,
            })
            .expect("Failed to store file list columns");

        instance.append(&column_view);
        self.column_view.set(column_view.clone()).unwrap();

        let instance_weak = instance.downgrade();
        column_view.sorter().unwrap().connect_changed(move |_, _| {
            if let Some(this) = instance_weak.upgrade() {
                this.sync_sort_model_sorter();
            }
        });

        // Listen for model changes to hook up selected change notification
        let instance_weak = instance.downgrade();
        column_view.connect_model_notify(move |cv| {
            if let Some(model) = cv.model() {
                if let Ok(selection_model) = model.downcast::<gtk4::SingleSelection>() {
                    let instance_weak = instance_weak.clone();
                    selection_model.connect_selected_notify(move |_| {
                        if let Some(this) = instance_weak.upgrade() {
                            if let Some(cb) = &*this.imp().selection_changed_callback.borrow() {
                                cb();
                            }
                        }
                    });
                }
            }
        });
    }
}

impl WidgetImpl for FileViewImp {}

impl BoxImpl for FileViewImp {}

impl FileViewImp {}
