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
    config::config,
    file_view::{Column, Cursor},
    image::draw::draw_text,
};
use gtk4::ListStore;
use std::{
    cell::RefCell,
    fs, io,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use super::{Backend, Target};

pub struct Bookmarks {
    store: ListStore,
    parent_backend: RefCell<Box<dyn Backend>>,
    parent_target: Target,
}

impl Bookmarks {
    pub fn new(parent_backend: Box<dyn Backend>, parent_target: Target) -> Self {
        Bookmarks {
            store: Self::create_store(),
            parent_backend: parent_backend.into(),
            parent_target,
        }
    }

    fn read_directory(store: &ListStore) -> io::Result<()> {
        let config = config();
        for entry in &config.bookmarks {
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
            store.insert_with_values(
                None,
                &[
                    (Column::Cat as u32, &cat.id()),
                    (Column::Icon as u32, &cat.icon()),
                    (Column::Name as u32, &entry.name),
                    (Column::Folder as u32, &entry.folder),
                    (Column::Size as u32, &file_size),
                    (Column::Modified as u32, &modified),
                ],
            );
        }
        Ok(())
    }

    fn create_store() -> ListStore {
        let store = Column::empty_store();
        match Self::read_directory(&store) {
            Ok(()) => (),
            Err(e) => {
                println!("read_dir failed {:?}", e);
            }
        }
        store
    }
}

impl Backend for Bookmarks {
    fn class_name(&self) -> &str {
        "Bookmarks"
    }

    fn is_bookmarks(&self) -> bool {
        true
    }

    fn path(&self) -> PathBuf {
        Path::new("bookmarks").into()
    }

    fn store(&self) -> ListStore {
        self.store.clone()
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

    fn image(&self, cursor: &Cursor, _: &ImageParams) -> Image {
        let folder = cursor.folder();
        let folder_lower = folder.to_lowercase();
        let cat = if folder_lower.ends_with(".zip") || folder_lower.ends_with(".rar") {
            Category::Archive
        } else {
            Category::Folder
        };
        draw_text(&cat.name(), &folder, cat.colors())
    }
}
