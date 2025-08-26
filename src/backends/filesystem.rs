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

use super::{Image, ImageParams};
use crate::{
    category::Category,
    error::MviewResult,
    file_view::{
        model::{BackendRef, ItemRef, Reference, Row},
        Cursor, Direction,
    },
    image::provider::{image_rs::RsImageLoader, internal::InternalImageLoader, ImageLoader},
    util::path_to_filename,
};
use image::DynamicImage;
use regex::Regex;
use std::{
    fs::{metadata, read_dir, rename},
    io,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use super::{Backend, Target};

pub struct FileSystem {
    directory: PathBuf,
    store: Vec<Row>,
}

impl FileSystem {
    pub fn new(directory: &Path) -> Self {
        FileSystem {
            directory: directory.into(),
            store: Self::read_directory(directory).unwrap_or_default(),
        }
    }

    fn read_directory(current_dir: &Path) -> io::Result<Vec<Row>> {
        let mut result = Vec::new();
        for entry in read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path_to_filename(&path);

            if filename.starts_with('.') {
                continue;
            }

            let metadata = match metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    println!("{filename}: Err = {e:?}");
                    continue;
                }
            };

            let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
            let modified = if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                duration.as_secs()
            } else {
                0
            };
            let size = metadata.len();

            let cat = Category::determine(&path, metadata.is_dir());

            let row = Row {
                category: cat.id(),
                name: filename.to_string(),
                size,
                modified,
                index: Default::default(),
                icon: cat.icon().to_string(),
                folder: Default::default(),
            };

            result.push(row);
        }
        Ok(result)
    }

    pub fn get_thumbnail(src: &Reference) -> MviewResult<DynamicImage> {
        if let (BackendRef::FileSystem(directory), ItemRef::String(name)) = src.as_tuple() {
            let filename = directory.join(name);
            if let Some(image) = InternalImageLoader::thumb_from_file(&filename) {
                Ok(image)
            } else {
                let thumb_filename = name.replace(".lo.", ".").replace(".hi.", ".") + ".mthumb";
                let thumb_path = directory.join(".mview").join(thumb_filename);
                if Path::new(&thumb_path).exists() {
                    RsImageLoader::dynimg_from_file(&thumb_path)
                } else {
                    let path = directory.join(name);
                    let image = RsImageLoader::dynimg_from_file(&path)?;
                    let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
                    // ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
                    Ok(image)
                }
            }
        } else {
            Err("invalid reference".into())
        }
    }
}

impl Backend for FileSystem {
    fn class_name(&self) -> &str {
        "FileSystem"
    }

    fn path(&self) -> PathBuf {
        self.directory.clone()
    }

    fn store(&self) -> &Vec<Row> {
        &self.store
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
            Some((
                Box::new(FileSystem::new(parent)),
                Target::Name(path_to_filename(&self.directory)),
            ))
        } else {
            None
        }
    }

    fn image(&self, item: &ItemRef, _: &ImageParams) -> Image {
        let filename = self.directory.join(item.str());
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
                println!("Failed to rename {filename} to {new_filename}: {e:?}");
                false
            }
        }
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::FileSystem(self.directory.clone())
    }

    fn item_ref(&self, cursor: &Cursor) -> ItemRef {
        ItemRef::String(cursor.name())
    }

    fn reload(&self) -> Option<Box<dyn Backend>> {
        let directory = &self.directory;
        Some(Box::new(FileSystem {
            directory: directory.into(),
            store: Self::read_directory(directory).unwrap_or_default(),
        }))
    }
}
