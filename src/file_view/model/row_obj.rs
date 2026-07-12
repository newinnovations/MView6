use glib::Object;
use gtk4::glib;

use crate::classification::FileClassification;

// Cloning a gio::ListStore is a very cheap O(1) operation. It simply creates a new Rust wrapper around
// the underlying C pointer and increments the reference count of the object.
pub type FileStore = gio::ListStore;

glib::wrapper! {
    pub struct FileRow(ObjectSubclass<super::row_imp::FileRow>);
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

    pub fn classification(&self) -> FileClassification {
        FileClassification::new(self.file_type(), self.preference())
    }
}
