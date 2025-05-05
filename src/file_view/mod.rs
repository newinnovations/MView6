// MView6 -- Opiniated image and pdf browser written in Rust and GTK4
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

use glib::{object::Cast, subclass::types::ObjectSubclassIsExt};
use gtk4::{
    glib,
    prelude::{TreeModelExt, TreeSortableExtManual, TreeViewExt},
    ListStore, TreeViewColumn,
};
pub use imp::{
    cursor::{Cursor, TreeModelMviewExt},
    model::{Columns, Direction, Filter, Selection},
    sort::Sort,
};

glib::wrapper! {
pub struct FileView(ObjectSubclass<imp::FileViewImp>)
    @extends gtk4::Widget, gtk4::TreeView, gtk4::Scrollable;
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

    pub fn goto(&self, selection: &Selection) -> bool {
        // println!("Goto {:?}", selection);
        if let Some(store) = self.store() {
            if let Some(iter) = store.iter_first() {
                loop {
                    let found = match selection {
                        Selection::Name(filename) => *filename == store.name(&iter),
                        Selection::Index(index) => *index == store.index(&iter),
                        Selection::None => true,
                    };
                    if found {
                        let tp = store.path(&iter); //.unwrap_or_default();
                        self.set_cursor(&tp, None::<&TreeViewColumn>, false);
                        return true;
                    }
                    if !store.iter_next(&iter) {
                        return false;
                    }
                }
            }
        }
        false
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

    pub fn set_extended(&self, extended: bool) {
        self.imp().set_extended(extended);
    }
}
