use glib::prelude::*;
use glib::subclass::prelude::*;
use std::cell::{Cell, RefCell};

use crate::classification::{FileType, Preference};

#[derive(Debug, Default, glib::Properties)]
#[properties(wrapper_type = super::FileRow)] // Links to the public struct wrapper
pub struct FileRow {
    #[property(get, set = Self::set_file_type, builder(FileType::Unsupported))]
    pub file_type: Cell<FileType>,

    #[property(get, set)]
    pub name: RefCell<String>,

    #[property(get, set)]
    pub folder: RefCell<String>,

    #[property(get, set)]
    pub size: Cell<u64>,

    #[property(get, set)]
    pub modified: Cell<u64>,

    #[property(get, set = Self::set_preference, builder(Preference::Normal))]
    pub preference: Cell<Preference>,

    #[property(get, set)]
    pub index: Cell<u64>,

    #[property(get, set = Self::set_trash)]
    pub trash: Cell<bool>,

    // Derived properties file_icon, pref_icon and pref_icon_visible are not settable directly,
    // but are computed based on other properties: file_type, preference and trash
    #[property(get)]
    pub file_icon: RefCell<String>,

    #[property(get)]
    pub pref_icon: RefCell<String>,

    #[property(get)]
    pub pref_icon_visible: Cell<bool>,
}

// Register the struct as a GObject subclass
#[glib::object_subclass]
impl ObjectSubclass for FileRow {
    const NAME: &'static str = "mv6FileRow";
    type Type = super::FileRow;
    type ParentType = glib::Object;
}

// Implement standard GObject behavior hooks
#[glib::derived_properties]
impl ObjectImpl for FileRow {}

impl FileRow {
    fn set_file_type(&self, file_type: FileType) {
        self.file_type.set(file_type);
        self.file_icon.replace(file_type.icon().to_string());
        self.obj().notify("file-type");
        self.obj().notify("file-icon");
    }

    fn set_preference(&self, preference: Preference) {
        if self.preference.get() != preference {
            self.preference.set(preference);
            self.update_preficon();
            self.obj().notify("preference");
        }
    }

    fn set_trash(&self, to_trash: bool) {
        if self.trash.get() != to_trash {
            self.trash.set(to_trash);
            self.update_preficon();
            self.obj().notify("trash");
        }
    }

    fn update_preficon(&self) {
        println!("Updating preficon {}", self.name.borrow());
        if self.trash.get() {
            self.pref_icon.replace("mv6-trash".to_string());
            self.pref_icon_visible.set(true);
        } else {
            let preference = self.preference.get();
            self.pref_icon.replace(preference.icon().to_string());
            self.pref_icon_visible.set(preference.show_icon());
        }
        self.obj().notify("pref-icon");
        self.obj().notify("pref-icon-visible");
    }
}
