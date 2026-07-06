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

use std::{collections::HashSet, fmt, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::classification::{FileClassification, FileType, Preference};

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum Direction {
    Up = 0,
    Down,
}

pub type FilterSet = (HashSet<FileType>, HashSet<Preference>);

#[derive(Debug, Default)]
pub enum Filter {
    #[default]
    None,
    Image,
    Liked,
    Container,
    Set(FilterSet),
}

impl Filter {
    pub fn full_set() -> Self {
        Self::Set((FileType::all(), Preference::all()))
    }

    pub fn matches(&self, category: FileClassification) -> bool {
        match self {
            Self::None => true,
            Self::Image => category.file_type == FileType::Image,
            Self::Liked => category.preference == Preference::Liked,
            Self::Container => {
                category.file_type == FileType::Folder
                    || category.file_type == FileType::Archive
                    || category.file_type == FileType::Document
            }
            Self::Set((ref c_set, ref f_set)) => {
                c_set.contains(&category.file_type) && f_set.contains(&category.preference)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Column {
    // First 4 need to be in the order on screen
    FileType = 0,
    Name,
    Size,
    Modified,
    Index,
    ContentIcon,
    PrefIcon,
    ShowPrefIcon,
    Folder,
}

#[derive(Debug, Clone)]
pub struct Row {
    pub file_type: u32,
    pub name: String,
    pub size: u64,
    pub modified: u64,
    index: u64,
    content_icon: String,
    preference_icon: String,
    show_preference_icon: bool,
    to_trash: bool,
    folder: String,
}

impl Row {
    pub fn new(classification: FileClassification, name: String, size: u64, modified: u64) -> Self {
        Self::new_folder_index(classification, name, size, modified, 0, Default::default())
    }

    pub fn new_index(
        classification: FileClassification,
        name: String,
        size: u64,
        modified: u64,
        index: u64,
    ) -> Self {
        Self::new_folder_index(
            classification,
            name,
            size,
            modified,
            index,
            Default::default(),
        )
    }

    pub fn new_folder_index(
        classification: FileClassification,
        name: String,
        size: u64,
        modified: u64,
        index: u64,
        folder: String,
    ) -> Self {
        Row {
            file_type: classification.file_type_id(),
            name,
            size,
            modified,
            index,
            content_icon: classification.file_type_icon().to_string(),
            preference_icon: classification.preference_icon().to_string(),
            show_preference_icon: classification.show_preference_icon(),
            to_trash: false,
            folder,
        }
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn content_icon(&self) -> &str {
        &self.content_icon
    }

    pub fn preference_icon(&self) -> &str {
        if self.to_trash {
            "mv6-trash"
        } else {
            &self.preference_icon
        }
    }

    pub fn show_preference_icon(&self) -> bool {
        self.show_preference_icon || self.to_trash
    }

    pub fn folder(&self) -> &str {
        &self.folder
    }

    pub fn set_preference(&mut self, preference: Preference) {
        self.preference_icon = preference.icon().to_string();
        self.show_preference_icon = preference.show_icon();
    }

    pub fn to_trash(&self) -> bool {
        self.to_trash
    }

    pub fn set_to_trash(&mut self, to_trash: bool) {
        self.to_trash = to_trash;
    }
}

pub mod file_row {
    use super::Row;
    use glib::subclass::types::ObjectSubclassIsExt;
    use gtk4::glib;

    glib::wrapper! {
        pub struct FileRow(ObjectSubclass<imp::FileRow>);
    }

    impl FileRow {
        pub fn new(row: Row) -> Self {
            let obj: Self = glib::Object::builder().build();
            *obj.imp().row.borrow_mut() = Some(row);
            obj
        }

        pub fn row(&self) -> std::cell::Ref<'_, Option<Row>> {
            self.imp().row.borrow()
        }

        pub fn row_mut(&self) -> std::cell::RefMut<'_, Option<Row>> {
            self.imp().row.borrow_mut()
        }
    }

    mod imp {
        use super::Row;
        use gtk4::glib;
        use gtk4::subclass::prelude::*;
        use std::cell::RefCell;

        #[derive(Default)]
        pub struct FileRow {
            pub row: RefCell<Option<Row>>,
        }

        #[glib::object_subclass]
        impl ObjectSubclass for FileRow {
            const NAME: &'static str = "FileRow";
            type Type = super::FileRow;
            type ParentType = glib::Object;
        }

        impl ObjectImpl for FileRow {}
    }
}

pub use file_row::FileRow;

use gtk4::gio;

impl Column {
    pub fn empty_store() -> gio::ListStore {
        gio::ListStore::new::<FileRow>()
    }

    pub fn store(index: &[Row]) -> gio::ListStore {
        let store = Self::empty_store();
        for row in index {
            store.append(&FileRow::new(row.clone()));
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
    pub category: FileClassification,
    pub name: String,
    pub reference: Reference,
}

impl Entry {
    pub fn new(category: FileClassification, name: &str, reference: Reference) -> Self {
        Entry {
            category,
            name: name.to_string(),
            reference,
        }
    }

    pub fn preference(&self) -> Preference {
        self.category.preference
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            category: Default::default(),
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
