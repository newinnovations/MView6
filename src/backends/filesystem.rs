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

use super::{Image, ImageParams};
use crate::{
    category::Category,
    error::MviewResult,
    file_view::{Column, Cursor, Direction},
    image::provider::{image_rs::RsImageLoader, internal::InternalImageLoader, ImageLoader},
};
use gtk4::ListStore;
use image::DynamicImage;
use regex::Regex;
use std::{
    fs::{metadata, read_dir, rename},
    io,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use super::{
    thumbnail::{TEntry, TReference},
    Backend, Target,
};

pub struct FileSystem {
    directory: PathBuf,
    store: ListStore,
}

impl FileSystem {
    pub fn new(directory: &Path) -> Self {
        FileSystem {
            directory: directory.into(),
            store: Self::create_store(directory),
        }
    }

    fn read_directory(store: &ListStore, current_dir: &Path) -> io::Result<()> {
        for entry in read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();

            if filename.starts_with('.') {
                continue;
            }

            let metadata = match metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    println!("{}: Err = {:?}", filename, e);
                    continue;
                }
            };

            let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
            let modified = if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                duration.as_secs()
            } else {
                0
            };
            let file_size = metadata.len();

            let cat = Category::determine(&path, metadata.is_dir());

            store.insert_with_values(
                None,
                &[
                    (Column::Cat as u32, &cat.id()),
                    (Column::Icon as u32, &cat.icon()),
                    (Column::Name as u32, &filename),
                    (Column::Size as u32, &file_size),
                    (Column::Modified as u32, &modified),
                ],
            );
        }
        Ok(())
    }

    fn create_store(directory: &Path) -> ListStore {
        let store = Column::empty_store();
        match Self::read_directory(&store, directory) {
            Ok(()) => (),
            Err(e) => {
                println!("read_dir failed {:?}", e);
            }
        }
        store
    }

    pub fn get_thumbnail(src: &TFileReference) -> MviewResult<DynamicImage> {
        if let Some(image) = InternalImageLoader::thumb_from_file(&src.path()) {
            Ok(image)
        } else {
            let thumb_filename = src.filename.replace(".lo.", ".").replace(".hi.", ".") + ".mthumb";
            let thumb_path = src.directory.join(".mview").join(thumb_filename);
            if Path::new(&thumb_path).exists() {
                RsImageLoader::dynimg_from_file(&thumb_path)
            } else {
                let path = src.directory.join(&src.filename);
                let image = RsImageLoader::dynimg_from_file(&path)?;
                let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
                // ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
                Ok(image)
            }
        }
    }
}

impl Backend for FileSystem {
    fn class_name(&self) -> &str {
        "FileSystem"
    }

    fn is_container(&self) -> bool {
        true
    }

    fn path(&self) -> PathBuf {
        self.directory.clone()
    }

    fn store(&self) -> ListStore {
        self.store.clone()
    }

    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        let category = cursor.category();
        if category == Category::Folder
            || category == Category::Archive
            || category == Category::Document
        {
            Some(<dyn Backend>::new(&self.directory.join(cursor.name())))
        } else {
            None
        }
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        if let Some(parent) = self.directory.parent() {
            let my_name = self
                .directory
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            Some((Box::new(FileSystem::new(parent)), Target::Name(my_name)))
        } else {
            None
        }
    }

    fn image(&self, cursor: &Cursor, _: &ImageParams) -> Image {
        let filename = self.directory.join(cursor.name());
        ImageLoader::image_from_file(&filename)
    }

    fn favorite(&self, cursor: &Cursor, direction: Direction) -> bool {
        let cat = cursor.category();
        if cat != Category::Image && cat != Category::Favorite && cat != Category::Trash {
            return false;
        }

        let filename = cursor.name();
        let re = Regex::new(r"\.([^\.]+)$").unwrap();
        let (new_filename, new_category) = if matches!(direction, Direction::Up) {
            if filename.contains(".hi.") {
                return true;
            } else if filename.contains(".lo.") {
                (filename.replace(".lo", ""), Category::Image)
            } else {
                (
                    re.replace(&filename, ".hi.$1").to_string(),
                    Category::Favorite,
                )
            }
        } else if filename.contains(".lo.") {
            return true;
        } else if filename.contains(".hi.") {
            (filename.replace(".hi", ""), Category::Image)
        } else {
            (re.replace(&filename, ".lo.$1").to_string(), Category::Trash)
        };
        dbg!(&self.directory, &filename, &new_filename);
        match rename(
            self.directory.join(&filename),
            self.directory.join(&new_filename),
        ) {
            Ok(()) => {
                cursor.update(new_category, &new_filename);
                true
            }
            Err(e) => {
                println!("Failed to rename {filename} to {new_filename}: {:?}", e);
                false
            }
        }
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        let name = &cursor.name();
        TEntry::new(
            cursor.category(),
            name,
            TReference::FileReference(TFileReference::new(&self.directory, name)),
        )
    }
}

#[derive(Debug, Clone)]
pub struct TFileReference {
    directory: PathBuf,
    filename: String,
}

impl TFileReference {
    pub fn new(directory: &Path, filename: &str) -> Self {
        TFileReference {
            directory: directory.into(),
            filename: filename.to_string(),
        }
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn path(&self) -> PathBuf {
        let p = Path::new(&self.directory);
        p.join(&self.filename)
    }
}
