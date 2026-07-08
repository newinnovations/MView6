mod imp;
mod reference;

use glib::Object;
use gtk4::glib;

use crate::classification::FileClassification;

pub use reference::{BackendRef, Column, Direction, Entry, Filter, ItemRef, Reference, Target};

type FileStore = gio::ListStore;

glib::wrapper! {
    pub struct FileRow(ObjectSubclass<imp::FileRow>);
}

impl FileRow {
    pub fn new(classification: FileClassification, name: String, size: u64, modified: u64) -> Self {
        Self::new_object(
            &name,
            size,
            modified,
            classification,
            0,
            Default::default(),
            false,
        )
    }

    pub fn new_index(
        classification: FileClassification,
        name: String,
        size: u64,
        modified: u64,
        index: u64,
    ) -> Self {
        Self::new_object(
            &name,
            size,
            modified,
            classification,
            index,
            Default::default(),
            false,
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
        Self::new_object(&name, size, modified, classification, index, &folder, false)
    }

    pub fn new_object(
        name: &str,
        size: u64,
        modified: u64,
        classification: FileClassification,
        index: u64,
        folder: &str,
        trash: bool,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("size", size)
            .property("modified", modified)
            .property("preference", classification.preference)
            .property("file-type", classification.file_type)
            .property("index", index)
            .property("folder", folder)
            .property("trash", trash)
            .build()
    }

    pub fn empty_store() -> FileStore {
        FileStore::new::<Self>()
    }

    pub fn store(index: &[Self]) -> FileStore {
        let store = Self::empty_store();
        for row in index {
            store.append(row);
        }
        store
    }
}
