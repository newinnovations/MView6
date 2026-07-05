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

use glib::{
    clone, idle_add_local, object::Cast, subclass::types::ObjectSubclassIsExt, ControlFlow,
};
use gtk4::{
    glib,
    prelude::{TreeModelExt, TreeModelExtManual, TreeSortableExtManual, TreeViewExt},
    ListStore, SortColumn, SortType, TreeIter, TreeViewColumn,
};

use crate::{
    file_view::{Column, Cursor, Direction, Filter, Target, TreeModelMviewExt},
    window::MViewWindow,
};
glib::wrapper! {
pub struct FileView(ObjectSubclass<super::FileViewImp>)
    @extends gtk4::Widget, gtk4::TreeView,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Scrollable;
}

impl FileView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for FileView {
    fn default() -> Self {
        Self::new()
    }
}

impl FileView {
    pub fn store(&self) -> Option<ListStore> {
        self.model()
            .and_then(|tree_model| tree_model.downcast::<ListStore>().ok())
    }

    pub fn current(&self) -> Option<Cursor> {
        let (tree_path, _) = self.cursor();
        if let Some(store) = self.store() {
            if let Some(path) = tree_path {
                store
                    .iter(&path)
                    .map(|iter| Cursor::new(store, iter, *path.indices().first().unwrap_or(&0)))
            } else {
                store.iter_first().map(|iter| Cursor::new(store, iter, 0))
            }
        } else {
            None
        }
    }

    /// Helper for goto function
    ///
    fn goto_iter(&self, window: &MViewWindow, store: &ListStore, iter: &TreeIter) {
        let tp = store.path(iter);
        let window = window.imp();
        let skip_loading = window.skip_loading.get();
        if skip_loading {
            // do not delay, we need the result now because the final goto which will come later
            self.set_cursor(&tp, None::<&TreeViewColumn>, false);
        } else {
            let open_container = window.open_container.get();
            if open_container {
                // do not delay, we need the result now because the final goto which will come later
                window.skip_loading.set(true);
                self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                window.open_container.set(false);
                window.skip_loading.set(false);
                window.dir_enter();
            } else {
                // This is the final goto: delay navigation so the file_view can render on screen
                // before executing set_cursor.
                // Capture store + iter (not the pre-computed TreePath) so that the path is
                // re-derived inside the callback, correctly reflecting any sort that may have
                // happened between enqueue and execution.
                let store = store.clone();
                let iter = *iter;
                idle_add_local(clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        // Re-derive path from iter to handle any sort that occurred
                        // between enqueue and execution.
                        let tp = store.path(&iter);
                        this.set_cursor(&tp, None::<&TreeViewColumn>, false);
                        ControlFlow::Break
                    }
                ));
            }
        }
    }

    /// Goto an entry in the list (files, pages, etc). We do this delayed using idle_add_local,
    /// so the file_view can render on screen before executing set_cursor. In some cases we go
    /// through several goto operations before reaching the final (with the `skip_loading` and
    /// `open_container` flags), in those cases do not delay as the file_view does not yet
    /// contain the final/desired content.
    ///
    /// If `target` is `First` or `Last`, we try to honor the `filter` argument. If the none of
    /// the items match the filter, we go to the actual first or last item.
    ///
    /// If a `Name` or `Index` target is not found, we will select the last item.
    ///
    /// Ignores empty lists.
    ///
    /// Gets called via:
    /// - MViewWindowImp::set_backend(.. goto: &target ..)
    /// - MViewWindowImp::reload(.. target: &Target) if the reload does not trigger a new backend
    /// - MViewWindowImp::slidshow_go_next to go to `First` item
    ///
    pub fn goto(&self, target: &Target, filter: &Filter, window: &MViewWindow) {
        // println!("fileview::goto {:?}", target);
        if let Some(store) = self.store() {
            let n = store.iter_n_children(None);
            if n < 1 {
                return;
            }
            let starting_point = if *target == Target::Last {
                store.iter_nth_child(None, n - 1)
            } else {
                store.iter_first()
            };
            if let Some(mut iter) = starting_point {
                loop {
                    if match target {
                        Target::Name(filename) => *filename == store.name(&iter),
                        Target::Index(index) => *index == store.index(&iter),
                        _ => filter.matches(store.category(&iter)),
                    } {
                        // Found what we are looking for
                        self.goto_iter(window, &store, &iter);
                        return;
                    }
                    let has_next = if *target == Target::Last {
                        store.iter_previous(&mut iter)
                    } else {
                        store.iter_next(&mut iter)
                    };
                    if !has_next {
                        break;
                    }
                }
            }
            // We did not find what we are looking for
            let fallback = if *target == Target::First {
                store.iter_first()
            } else {
                store.iter_nth_child(None, n - 1)
            };
            if let Some(iter) = fallback {
                self.goto_iter(window, &store, &iter);
            }
        }
    }

    // RUST_BUG: navigate_item_bool sometimes gets optimized out if the result is not used.
    // We need to provide an alternative that does not have a return value, to prevent the
    // function from being optimized out in those cases.
    pub fn navigate_item(&self, direction: Direction, filter: &Filter, count: u32) {
        if let Some(mut current) = self.current() {
            let treepath = current.navigate(direction, filter, count);
            if let Some(tree_path) = &treepath {
                self.set_cursor(tree_path, None::<&TreeViewColumn>, false);
            }
        }
    }

    pub fn navigate_item_bool(&self, direction: Direction, filter: &Filter, count: u32) -> bool {
        if let Some(mut current) = self.current() {
            let treepath = current.navigate(direction, filter, count);
            if let Some(tree_path) = &treepath {
                self.set_cursor(tree_path, None::<&TreeViewColumn>, false);
                return true;
            }
        }
        false
    }

    pub fn set_unsorted(&self) {
        if let Some(store) = self.store() {
            store.set_unsorted();
        }
    }

    pub fn set_sortable(&self, sortable: bool) {
        self.set_headers_clickable(sortable);
        for (i, column) in self.columns().iter().enumerate() {
            column.set_clickable(sortable);
            column.set_sort_column_id(if sortable { i as i32 } else { -1 });
        }
    }

    pub fn set_extended(&self, extended: bool) {
        self.imp().set_extended(extended);
    }

    pub fn change_sort(&self, sort_col: Column) {
        if let Some(store) = self.store() {
            let new_sort_column = SortColumn::Index(sort_col as u32);
            let current_sort = store.sort_column_id();
            let new_direction = match current_sort {
                Some((current_column, current_direction)) => {
                    if current_column.eq(&new_sort_column) {
                        match current_direction {
                            SortType::Ascending => SortType::Descending,
                            _ => SortType::Ascending,
                        }
                    } else {
                        SortType::Ascending
                    }
                }
                None => SortType::Ascending,
            };
            store.set_sort_column_id(new_sort_column, new_direction);
        }
    }
}
