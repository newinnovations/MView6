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

mod imp;

use glib::{
    clone, idle_add_local, object::Cast, subclass::types::ObjectSubclassIsExt, ControlFlow,
};
use gtk4::{
    glib,
    prelude::{TreeModelExt, TreeSortableExtManual, TreeViewExt},
    ListStore, TreeIter, TreeViewColumn,
};
pub use imp::{
    cursor::{Cursor, TreeModelMviewExt},
    model::{Column, Direction, Filter, Target},
    sort::Sort,
};

use crate::window::MViewWindow;
glib::wrapper! {
pub struct FileView(ObjectSubclass<imp::FileViewImp>)
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
    fn store(&self) -> Option<ListStore> {
        if let Some(model) = self.model() {
            model.downcast::<ListStore>().ok()
        } else {
            None
        }
    }

    pub fn current(&self) -> Option<Cursor> {
        let (tree_path, _) = self.cursor();
        if let Some(store) = self.store() {
            if let Some(path) = tree_path {
                store.iter(&path).map(|iter| Cursor {
                    store,
                    iter,
                    position: *path.indices().first().unwrap_or(&0),
                })
            } else {
                store.iter_first().map(|iter| Cursor {
                    store,
                    iter,
                    position: 0,
                })
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
                // this is the final goto: delay navigation so the file_view can render on screen
                // before executing set_cursor
                idle_add_local(clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.set_cursor(&tp, None::<&TreeViewColumn>, false);
                        ControlFlow::Break
                    }
                ));
            }
        }
    }

    /// Goto an entry in the in list (files, pages, etc). We do this delayed using idle_add_local,
    /// so the file_view can render on screen before executing set_cursor. In some cases we go
    /// through several goto operations before reaching the final (with skip_loading and
    /// open_container), in those cases do not delay as the file_view does not yet contain the
    /// desired contents.
    ///
    /// If not found, we will select the last item. Ignores empty lists.
    ///
    /// Gets called via:
    /// - MViewWindowImp::set_backend(.. goto: &target ..)
    ///
    pub fn goto(&self, target: &Target, window: &MViewWindow) {
        // println!("fileview::goto {:?}", target);
        if let Some(store) = self.store() {
            let n = store.iter_n_children(None);
            if n < 1 {
                return;
            }
            if *target != Target::Last {
                if let Some(iter) = store.iter_first() {
                    loop {
                        let found = match target {
                            Target::Name(filename) => *filename == store.name(&iter),
                            Target::Index(index) => *index == store.index(&iter),
                            _ => true,
                        };
                        if found {
                            self.goto_iter(window, &store, &iter);
                            return;
                        }
                        if !store.iter_next(&iter) {
                            break;
                        }
                    }
                }
            }
            if let Some(iter) = store.iter_nth_child(None, n - 1) {
                self.goto_iter(window, &store, &iter);
            }
        }
    }

    pub fn home(&self) {
        if let Some(store) = self.store() {
            if let Some(iter) = store.iter_first() {
                let tp = store.path(&iter);
                self.set_cursor(&tp, None::<&TreeViewColumn>, false);
            }
        }
    }

    pub fn end(&self) {
        if let Some(store) = self.store() {
            let num_items = store.iter_n_children(None);
            if num_items > 1 {
                if let Some(iter) = store.iter_nth_child(None, num_items - 1) {
                    let tp = store.path(&iter);
                    self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                }
            }
        }
    }

    pub fn navigate(&self, direction: Direction, filter: Filter, count: u32) {
        if let Some(current) = self.current() {
            if let Some(tree_path) = current.navigate(direction, filter, count) {
                self.set_cursor(&tree_path, None::<&TreeViewColumn>, false);
            }
        }
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
}
