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

use super::{Content, ImageParams};
use crate::{
    category::Category,
    config::config,
    file_view::{
        model::{BackendRef, ItemRef, Row},
        Cursor,
    },
    image::provider::ImageLoader,
};
use std::{
    cell::RefCell,
    fs, io,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use super::{Backend, Target};

pub struct Bookmarks {
    store: Vec<Row>,
    parent_backend: RefCell<Box<dyn Backend>>,
    parent_target: Target,
}

impl Bookmarks {
    pub fn new(parent_backend: Box<dyn Backend>, parent_target: Target) -> Self {
        Bookmarks {
            store: Self::read_bookmarks().unwrap_or_default(),
            parent_backend: parent_backend.into(),
            parent_target,
        }
    }

    fn read_bookmarks() -> io::Result<Vec<Row>> {
        let mut result = Vec::new();
        let config = config();
        for entry in &config.config_file.bookmarks {
            let metadata = match fs::metadata(&entry.folder) {
                Ok(m) => m,
                Err(e) => {
                    println!("{}: Err = {:?}", &entry.folder, e);
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
            let cat = Category::Folder;
            let row = Row {
                category: cat.id(),
                name: entry.name.clone(),
                size: file_size,
                modified,
                index: Default::default(),
                icon: cat.icon().to_string(),
                folder: entry.folder.clone(),
            };

            result.push(row);
        }
        Ok(result)
    }
}

impl Backend for Bookmarks {
    fn class_name(&self) -> &str {
        "Bookmarks"
    }

    fn path(&self) -> PathBuf {
        Path::new("bookmarks").into()
    }

    fn store(&self) -> &Vec<Row> {
        &self.store
    }

    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        Some(<dyn Backend>::new(Path::new(&cursor.folder())))
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        Some((
            self.parent_backend.replace(<dyn Backend>::none()),
            self.parent_target.clone(),
        ))
    }

    fn image(&self, item: &ItemRef, _: &ImageParams) -> Content {
        let path = Path::new(item.str());
        ImageLoader::image_from_file(path)
        // let cat = if folder_lower.ends_with(".zip") || folder_lower.ends_with(".rar") {
        //     Category::Archive
        // } else {
        //     Category::Folder
        // };
        // draw_text(&cat.name(), folder, cat.colors())
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::Bookmarks
    }

    fn item_ref(&self, cursor: &Cursor) -> ItemRef {
        ItemRef::String(cursor.folder())
    }
}
