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

use gtk4::glib::{
    self, clone, idle_add_local, object::Cast, subclass::types::ObjectSubclassIsExt, ControlFlow,
};
use gtk4::{gio, prelude::ListModelExt, SortType};

use super::model::file_row::FileRow;
use crate::{
    classification::{FileClassification, FileType, Preference},
    file_view::{Column, Cursor, Direction, Filter, Target},
    window::MViewWindow,
};

glib::wrapper! {
pub struct FileView(ObjectSubclass<super::FileViewImp>)
    @extends gtk4::Box, gtk4::Widget,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl FileView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn column_view(&self) -> &gtk4::ColumnView {
        self.imp().column_view.get().unwrap()
    }

    pub fn model(&self) -> Option<gtk4::SelectionModel> {
        self.column_view().model()
    }

    pub fn set_model<P: gtk4::glib::prelude::IsA<gtk4::SelectionModel>>(&self, model: Option<&P>) {
        self.column_view().set_model(model.map(|m| m.as_ref()));
    }

    pub fn sorter(&self) -> Option<gtk4::Sorter> {
        self.column_view().sorter()
    }

    pub fn model_sorter(&self) -> Option<gtk4::Sorter> {
        Some(self.imp().sort_model_sorter.get()?.clone().upcast())
    }

    pub fn sort_by_column(&self, col: Option<&gtk4::ColumnViewColumn>, order: SortType) {
        self.column_view().sort_by_column(col, order);
    }

    pub fn scroll_to(
        &self,
        index: u32,
        col: Option<&gtk4::ColumnViewColumn>,
        flags: gtk4::ListScrollFlags,
        scroll: Option<gtk4::ScrollInfo>,
    ) {
        self.column_view().scroll_to(index, col, flags, scroll);
    }

    pub fn connect_activate<F: Fn(&gtk4::ColumnView, u32) + 'static>(&self, f: F) {
        self.column_view().connect_activate(f);
    }
}

impl Default for FileView {
    fn default() -> Self {
        Self::new()
    }
}

impl FileView {
    fn apply_sort_model_sorter(&self, sort: Option<(Column, SortType)>) {
        let imp = self.imp();
        let sort_model_sorter = imp.sort_model_sorter.get().unwrap();
        while sort_model_sorter.n_items() > 0 {
            sort_model_sorter.remove(0);
        }

        let Some((sort_col, order)) = sort else {
            return;
        };

        let sorters = imp.sorters.get().unwrap();
        let primary = match (sort_col, order) {
            (Column::FileType, SortType::Ascending) => &sorters.category,
            (Column::FileType, SortType::Descending) => &sorters.category_desc,
            (Column::Name, SortType::Ascending) => &sorters.name,
            (Column::Name, SortType::Descending) => &sorters.name_desc,
            (Column::Size, SortType::Ascending) => &sorters.size,
            (Column::Size, SortType::Descending) => &sorters.size_desc,
            (Column::Modified, SortType::Ascending) => &sorters.date,
            (Column::Modified, SortType::Descending) => &sorters.date_desc,
            _ => return,
        };

        sort_model_sorter.append(primary.clone());
        if sort_col != Column::Name {
            sort_model_sorter.append(sorters.name.clone());
        }
        sort_model_sorter.append(sorters.index.clone());
    }

    pub(super) fn sync_sort_model_sorter(&self) {
        self.apply_sort_model_sorter(self.current_sort());
    }

    fn scroll_index_into_view(&self, index: u32) {
        let scroll = gtk4::ScrollInfo::new();
        scroll.set_enable_horizontal(false);
        scroll.set_enable_vertical(true);
        self.scroll_to(
            index,
            None::<&gtk4::ColumnViewColumn>,
            gtk4::ListScrollFlags::NONE,
            Some(scroll),
        );
    }

    pub fn store(&self) -> Option<gio::ListModel> {
        self.model()?
            .downcast::<gtk4::SingleSelection>()
            .ok()?
            .model()
    }

    pub fn source_store(&self) -> Option<gio::ListStore> {
        let model = self.store()?;
        if let Ok(sort_model) = model.clone().downcast::<gtk4::SortListModel>() {
            return sort_model.model()?.downcast::<gio::ListStore>().ok();
        }
        model.downcast::<gio::ListStore>().ok()
    }

    pub fn current(&self) -> Option<Cursor> {
        if let Some(store) = self.store() {
            let source_store = self.source_store();
            if let Some(selection_model) = self
                .model()
                .and_then(|m| m.downcast::<gtk4::SingleSelection>().ok())
            {
                let selected_idx = selection_model.selected();
                if selected_idx != gtk4::INVALID_LIST_POSITION {
                    if let Some(obj) = store.item(selected_idx) {
                        if let Ok(file_row) = obj.downcast::<FileRow>() {
                            return Some(Cursor::new(store, source_store, file_row, selected_idx));
                        }
                    }
                }
            }
            if store.n_items() > 0 {
                if let Some(obj) = store.item(0) {
                    if let Ok(file_row) = obj.downcast::<FileRow>() {
                        return Some(Cursor::new(store, source_store, file_row, 0));
                    }
                }
            }
        }
        None
    }

    pub fn select_index(&self, index: u32) {
        if let Some(selection_model) = self
            .model()
            .and_then(|m| m.downcast::<gtk4::SingleSelection>().ok())
        {
            let selection_unchanged = selection_model.selected() == index;
            selection_model.set_selected(index);
            if selection_unchanged {
                if let Some(cb) = &*self.imp().selection_changed_callback.borrow() {
                    cb();
                }
            }
            idle_add_local(clone!(
                #[weak(rename_to = this)]
                self,
                #[upgrade_or]
                ControlFlow::Break,
                move || {
                    this.scroll_index_into_view(index);
                    ControlFlow::Break
                }
            ));
        }
    }

    fn goto_idx(&self, window: &MViewWindow, idx: u32) {
        let window = window.imp();
        let skip_loading = window.skip_loading.get();
        if skip_loading {
            self.select_index(idx);
        } else {
            let open_container = window.open_container.get();
            if open_container {
                window.skip_loading.set(true);
                self.select_index(idx);
                window.open_container.set(false);
                window.skip_loading.set(false);
                window.dir_enter();
            } else {
                idle_add_local(clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.select_index(idx);
                        ControlFlow::Break
                    }
                ));
            }
        }
    }

    pub fn goto(&self, target: &Target, filter: &Filter, window: &MViewWindow) {
        if let Some(store) = self.store() {
            let n = store.n_items();
            if n < 1 {
                return;
            }
            let starting_idx = if *target == Target::Last { n - 1 } else { 0 };

            let mut idx = starting_idx;
            loop {
                if let Some(obj) = store.item(idx) {
                    if let Ok(file_row) = obj.downcast::<FileRow>() {
                        let row = file_row.row();
                        let row_ref = row.as_ref().unwrap();
                        let matches = match target {
                            Target::Name(filename) => *filename == row_ref.name,
                            Target::Index(index) => *index == row_ref.index(),
                            _ => {
                                let category = FileClassification::new(
                                    FileType::from(row_ref.file_type),
                                    Preference::from_icon(row_ref.preference_icon()),
                                );
                                filter.matches(category)
                            }
                        };
                        if matches {
                            self.goto_idx(window, idx);
                            return;
                        }
                    }
                }

                if *target == Target::Last {
                    if idx == 0 {
                        break;
                    }
                    idx -= 1;
                } else {
                    idx += 1;
                    if idx >= n {
                        break;
                    }
                }
            }

            // Fallback
            let fallback_idx = if *target == Target::First { 0 } else { n - 1 };
            self.goto_idx(window, fallback_idx);
        }
    }

    pub fn navigate_item(&self, direction: Direction, filter: &Filter, count: u32) {
        if let Some(mut current) = self.current() {
            if let Some(new_idx) = current.navigate(direction, filter, count) {
                self.select_index(new_idx);
            }
        }
    }

    pub fn navigate_item_bool(&self, direction: Direction, filter: &Filter, count: u32) -> bool {
        if let Some(mut current) = self.current() {
            if let Some(new_idx) = current.navigate(direction, filter, count) {
                self.select_index(new_idx);
                return true;
            }
        }
        false
    }

    pub fn set_unsorted(&self) {
        self.sort_by_column(None::<&gtk4::ColumnViewColumn>, SortType::Ascending);
        self.apply_sort_model_sorter(None);
    }

    pub fn set_sortable(&self, sortable: bool) {
        let imp = self.imp();
        if let Some(columns) = imp.columns.get() {
            if sortable {
                if let Some(sorters) = imp.sorters.get() {
                    columns.category.set_sorter(Some(&sorters.category));
                    columns.name.set_sorter(Some(&sorters.name));
                    columns.size.set_sorter(Some(&sorters.size));
                    columns.date.set_sorter(Some(&sorters.date));
                }
            } else {
                columns.category.set_sorter(None::<&gtk4::Sorter>);
                columns.name.set_sorter(None::<&gtk4::Sorter>);
                columns.size.set_sorter(None::<&gtk4::Sorter>);
                columns.date.set_sorter(None::<&gtk4::Sorter>);
            }
        }
    }

    pub fn set_extended(&self, extended: bool) {
        self.imp().set_extended(extended);
    }

    pub fn current_sort(&self) -> Option<(Column, SortType)> {
        let sorter = self.sorter()?;
        let cv_sorter = sorter.downcast::<gtk4::ColumnViewSorter>().ok()?;
        let col = cv_sorter.primary_sort_column()?;
        let order = cv_sorter.primary_sort_order();
        let cols = self.imp().columns.get()?;
        let sort_col = if col == cols.category {
            Column::FileType
        } else if col == cols.name {
            Column::Name
        } else if col == cols.size {
            Column::Size
        } else if col == cols.date {
            Column::Modified
        } else {
            return None;
        };
        Some((sort_col, order))
    }

    pub fn set_sort(&self, sort_col: Column, order: SortType) {
        let imp = self.imp();
        if let Some(cols) = imp.columns.get() {
            let col = match sort_col {
                Column::FileType => &cols.category,
                Column::Name => &cols.name,
                Column::Size => &cols.size,
                Column::Modified => &cols.date,
                _ => return,
            };
            self.sort_by_column(None::<&gtk4::ColumnViewColumn>, SortType::Ascending);
            self.sort_by_column(Some(col), order);
            self.apply_sort_model_sorter(Some((sort_col, order)));
        }
    }

    pub fn change_sort(&self, sort_col: Column) {
        if let Some((current_col, current_order)) = self.current_sort() {
            let new_order = if current_col == sort_col {
                match current_order {
                    SortType::Ascending => SortType::Descending,
                    _ => SortType::Ascending,
                }
            } else {
                if sort_col == Column::Modified {
                    SortType::Descending
                } else {
                    SortType::Ascending
                }
            };
            self.set_sort(sort_col, new_order);
        } else {
            let order = if sort_col == Column::Modified {
                SortType::Descending
            } else {
                SortType::Ascending
            };
            self.set_sort(sort_col, order);
        }
    }

    pub fn connect_selection_changed<F: Fn() + 'static>(&self, f: F) {
        *self.imp().selection_changed_callback.borrow_mut() = Some(Box::new(f));
    }
}
