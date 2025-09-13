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

use std::{fmt, path::PathBuf, str::FromStr};

use gtk4::{prelude::TreeSortableExtManual, ListStore};
use serde::{Deserialize, Serialize};

use super::cursor::TreeModelMviewExt;
use crate::category::Category;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct Row {
    pub category: u32,
    pub name: String,
    pub size: u64,
    pub modified: u64,
    pub index: u64,
    pub icon: String,
    pub folder: String,
}

impl Row {
    pub fn push(&self, store: &ListStore) {
        store.insert_with_values(
            None,
            &[
                (Column::Cat as u32, &self.category),
                (Column::Name as u32, &self.name),
                (Column::Size as u32, &self.size),
                (Column::Modified as u32, &self.modified),
                (Column::Index as u32, &self.index),
                (Column::Icon as u32, &self.icon),
                (Column::Folder as u32, &self.folder),
            ],
        );
    }
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

    pub fn store(index: &Vec<Row>) -> ListStore {
        let store = Self::empty_store();
        for row in index {
            row.push(&store);
        }
        store
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Target {
    First,
    Name(String),
    Index(u64),
    Last,
}

impl From<Reference> for Target {
    fn from(value: Reference) -> Self {
        match value.take_tuple() {
            (BackendRef::FileSystem(_), ItemRef::String(name)) => Target::Name(name),
            (BackendRef::MarArchive(_), ItemRef::Index(index)) => Target::Index(index),
            (BackendRef::RarArchive(_), ItemRef::String(name)) => Target::Name(name),
            (BackendRef::ZipArchive(_), ItemRef::Index(index)) => Target::Index(index),
            (BackendRef::Mupdf(_), ItemRef::Index(index)) => Target::Index(index),
            (BackendRef::Pdfium(_), ItemRef::Index(index)) => Target::Index(index),
            (_, _) => Target::First,
        }
    }
}

impl From<Entry> for Target {
    fn from(item: Entry) -> Self {
        item.reference.into()
    }
}

#[derive(Debug, Clone)]
pub struct Reference {
    pub backend: BackendRef,
    pub item: ItemRef,
}

impl Reference {
    pub fn as_tuple(&self) -> (&BackendRef, &ItemRef) {
        (&self.backend, &self.item)
    }
    pub fn take_tuple(self) -> (BackendRef, ItemRef) {
        (self.backend, self.item)
    }
    pub fn supports_bot(&self) -> bool {
        self.backend.supports_bot()
    }
}

impl Default for Reference {
    fn default() -> Self {
        Self {
            backend: BackendRef::None,
            item: ItemRef::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendRef {
    FileSystem(PathBuf),
    MarArchive(PathBuf),
    RarArchive(PathBuf),
    ZipArchive(PathBuf),
    Mupdf(PathBuf),
    Pdfium(PathBuf),
    Thumbnail, //(Box<Reference>),
    Bookmarks,
    None,
}

impl BackendRef {
    pub fn new(name: &str, path: PathBuf) -> Self {
        match name {
            "FileSystem" => BackendRef::FileSystem(path),
            "MarArchive" => BackendRef::MarArchive(path),
            "RarArchive" => BackendRef::RarArchive(path),
            "ZipArchive" => BackendRef::ZipArchive(path),
            "Mupdf" => BackendRef::Mupdf(path),
            "Pdfium" => BackendRef::Pdfium(path),
            "Thumbnail" => BackendRef::Thumbnail,
            "Bookmarks" => BackendRef::Bookmarks,
            _ => BackendRef::None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            BackendRef::FileSystem(_) => "FileSystem",
            BackendRef::MarArchive(_) => "MarArchive",
            BackendRef::RarArchive(_) => "RarArchive",
            BackendRef::ZipArchive(_) => "ZipArchive",
            BackendRef::Mupdf(_) => "Mupdf",
            BackendRef::Pdfium(_) => "Pdfium",
            BackendRef::Thumbnail => "Thumbnail",
            BackendRef::Bookmarks => "Bookmarks",
            BackendRef::None => "None",
        }
    }

    pub fn path(&self) -> &str {
        let p = match self {
            BackendRef::FileSystem(path_buf) => path_buf.to_str(),
            BackendRef::MarArchive(path_buf) => path_buf.to_str(),
            BackendRef::RarArchive(path_buf) => path_buf.to_str(),
            BackendRef::ZipArchive(path_buf) => path_buf.to_str(),
            BackendRef::Mupdf(path_buf) => path_buf.to_str(),
            BackendRef::Pdfium(path_buf) => path_buf.to_str(),
            BackendRef::Thumbnail => None,
            BackendRef::Bookmarks => None,
            BackendRef::None => None,
        };
        p.unwrap_or_default()
    }

    pub fn supports_bot(&self) -> bool {
        matches!(
            self,
            BackendRef::FileSystem(_)
                | BackendRef::MarArchive(_)
                | BackendRef::RarArchive(_)
                | BackendRef::ZipArchive(_)
                | BackendRef::Mupdf(_)
                | BackendRef::Pdfium(_)
        )
    }

    pub fn is_none(&self) -> bool {
        matches!(self, BackendRef::None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ItemRef {
    String(String),
    Index(u64),
    #[default]
    None,
}

impl ItemRef {
    pub fn new_from_row(backend: &BackendRef, row: &Row) -> Self {
        match backend {
            BackendRef::FileSystem(_) => ItemRef::String(row.name.clone()),
            BackendRef::MarArchive(_) => ItemRef::Index(row.index),
            BackendRef::RarArchive(_) => ItemRef::String(row.name.clone()),
            BackendRef::ZipArchive(_) => ItemRef::Index(row.index),
            BackendRef::Mupdf(_) => ItemRef::Index(row.index),
            BackendRef::Pdfium(_) => ItemRef::Index(row.index),
            BackendRef::Thumbnail => ItemRef::Index(row.index),
            BackendRef::Bookmarks => ItemRef::String(row.folder.clone()),
            BackendRef::None => ItemRef::None,
        }
    }

    pub fn str(&self) -> &str {
        match self {
            ItemRef::String(s) => s,
            ItemRef::Index(_) => {
                eprintln!("should not happen: requested str() from ItemRef::Index");
                ""
            }
            ItemRef::None => {
                eprintln!("should not happen: requested str() from ItemRef::None");
                ""
            }
        }
    }

    pub fn idx(&self) -> u64 {
        match self {
            ItemRef::Index(i) => *i,
            ItemRef::String(_) => {
                eprintln!("should not happen: requested idx() from ItemRef::String");
                0
            }
            ItemRef::None => {
                eprintln!("should not happen: requested idx() from ItemRef::None");
                0
            }
        }
    }

    pub fn to_string_repr(&self) -> String {
        self.to_string()
    }

    pub fn from_string_repr(s: &str) -> Result<Self, String> {
        s.parse()
    }

    pub fn is_none(&self) -> bool {
        matches!(self, ItemRef::None)
    }
}

impl fmt::Display for ItemRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemRef::String(s) => write!(f, "s:{}", s),
            ItemRef::Index(i) => write!(f, "i:{}", i),
            ItemRef::None => write!(f, "n"),
        }
    }
}

impl FromStr for ItemRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "n" {
            Ok(ItemRef::None)
        } else if let Some(stripped) = s.strip_prefix("s:") {
            Ok(ItemRef::String(stripped.to_string()))
        } else if let Some(stripped) = s.strip_prefix("i:") {
            match stripped.parse::<u64>() {
                Ok(index) => Ok(ItemRef::Index(index)),
                Err(_) => Err(format!("Invalid index: {}", stripped)),
            }
        } else {
            Err(format!("Invalid format: {}", s))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub category: Category,
    pub name: String,
    pub reference: Reference,
}

impl Entry {
    pub fn new(category: Category, name: &str, reference: Reference) -> Self {
        Entry {
            category,
            name: name.to_string(),
            reference,
        }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            category: Category::Unsupported,
            name: Default::default(),
            reference: Reference {
                backend: BackendRef::None,
                item: ItemRef::Index(0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_zoom_mode_string_conversion() {
        assert_send_sync::<Reference>();
        assert_send_sync::<BackendRef>();
        assert_send_sync::<ItemRef>();
    }

    #[test]
    fn test_string_serialization() {
        let string_ref = ItemRef::String("hello world".to_string());
        let serialized = string_ref.to_string();
        assert_eq!(serialized, "s:hello world");

        let deserialized: ItemRef = serialized.parse().unwrap();
        assert_eq!(deserialized, string_ref);
    }

    #[test]
    fn test_index_serialization() {
        let index_ref = ItemRef::Index(42);
        let serialized = index_ref.to_string();
        assert_eq!(serialized, "i:42");

        let deserialized: ItemRef = serialized.parse().unwrap();
        assert_eq!(deserialized, index_ref);
    }

    #[test]
    fn test_none_serialization() {
        let index_ref = ItemRef::None;
        let serialized = index_ref.to_string();
        assert_eq!(serialized, "n");

        let deserialized: ItemRef = serialized.parse().unwrap();
        assert_eq!(deserialized, index_ref);
    }

    #[test]
    fn test_error_cases() {
        assert!(ItemRef::from_str("invalid").is_err());
        assert!(ItemRef::from_str("i:not_a_number").is_err());
    }
}
