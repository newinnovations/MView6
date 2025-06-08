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

use gtk4::{prelude::TreeSortableExtManual, ListStore};

use super::cursor::TreeModelMviewExt;
use crate::backends::thumbnail::{TEntry, TReference};

#[derive(Debug)]
#[repr(i32)]
pub enum Direction {
    Up = 0,
    Down,
}

#[derive(Debug)]
#[repr(i32)]
pub enum Filter {
    None = 0,
    Image,
    Favorite,
    Container,
}

#[derive(Debug)]
#[repr(u32)]
pub enum Column {
    // First 4 need to be in the order on screen
    Cat = 0,
    Name,
    Size,
    Modified,
    Index,
    Icon,
    Folder,
}

impl Column {
    pub fn empty_store() -> ListStore {
        let col_types: [glib::Type; 7] = [
            glib::Type::U32,
            glib::Type::STRING,
            glib::Type::U64,
            glib::Type::U64,
            glib::Type::U64,
            glib::Type::STRING,
            glib::Type::STRING,
        ];
        let store = ListStore::new(&col_types);
        store.set_sort_func(
            gtk4::SortColumn::Index(Column::Cat as u32),
            |model, iter1, iter2| {
                let cat1 = model.category_id(iter1);
                let cat2 = model.category_id(iter2);
                let result = cat1.cmp(&cat2);
                if result.is_eq() {
                    let filename1 = model.name(iter1).to_lowercase();
                    let filename2 = model.name(iter2).to_lowercase();
                    filename1.cmp(&filename2)
                } else {
                    result
                }
                .into()
            },
        );
        store
    }
}

#[derive(Clone, Debug)]
pub enum Target {
    First,
    Name(String),
    Index(u64),
}

impl From<TReference> for Target {
    fn from(item: TReference) -> Self {
        match item {
            TReference::FileReference(file) => Target::Name(file.filename()),
            TReference::ZipReference(zip) => Target::Index(zip.index()),
            TReference::MarReference(zip) => Target::Index(zip.index()),
            TReference::RarReference(rar) => Target::Name(rar.selection()),
            TReference::DocReference(doc) => Target::Index(doc.index()),
            TReference::None => Target::First,
        }
    }
}

impl From<TEntry> for Target {
    fn from(item: TEntry) -> Self {
        item.reference.into()
    }
}
