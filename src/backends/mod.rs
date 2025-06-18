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

use std::{
    env,
    path::{Path, PathBuf},
};

use archive_mar::MarArchive;
use archive_rar::RarArchive;
use archive_zip::ZipArchive;
use async_channel::Sender;
use bookmarks::Bookmarks;
use document::{Document, PageMode};
use filesystem::FileSystem;
use gtk4::ListStore;
use none::NoneBackend;
use thumbnail::{Message, TEntry, Thumbnail};

use crate::{
    backends::thumbnail::model::TParent,
    file_view::{Cursor, Direction, Target},
    image::Image,
};

mod archive_mar;
mod archive_rar;
mod archive_zip;
mod bookmarks;
pub mod document;
pub mod filesystem;
mod none;
pub mod thumbnail;

pub struct ImageParams<'a> {
    pub sender: &'a Sender<Message>,
    pub page_mode: &'a PageMode,
}

#[allow(unused_variables)]
pub trait Backend {
    fn class_name(&self) -> &str;
    fn path(&self) -> PathBuf;
    fn store(&self) -> ListStore;
    fn favorite(&self, cursor: &Cursor, direction: Direction) -> bool {
        false
    }
    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        None
    }
    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        if let Some(parent) = self.path().parent() {
            let my_name = self
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            Some((Box::new(FileSystem::new(parent)), Target::Name(my_name)))
        } else {
            None
        }
    }
    fn image(&self, cursor: &Cursor, params: &ImageParams) -> Image;
    fn entry(&self, cursor: &Cursor) -> TEntry {
        Default::default()
    }
    fn is_container(&self) -> bool {
        false
    }
    fn is_bookmarks(&self) -> bool {
        false
    }
    fn is_thumbnail(&self) -> bool {
        false
    }
    fn is_doc(&self) -> bool {
        false
    }
    fn is_none(&self) -> bool {
        false
    }
    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Target)> {
        None
    }
    fn can_be_sorted(&self) -> bool {
        !(self.is_thumbnail() || self.is_doc())
    }
    // Only implemented by thumbnail backend, dummy here
    fn get_thumb_parent(&self) -> TParent {
        TParent {
            backend: <dyn Backend>::none(),
            target: Target::First,
            focus_pos: 0,
        }
    }
    // Only implemented by filesystem backend, dummy here
    fn reload(&self) -> Option<Box<dyn Backend>> {
        None
    }
    fn normalized_path(&self) -> PathBuf {
        let path = self.path();
        #[cfg(windows)]
        {
            // Remove the \\?\ prefix if present on Windows
            let path_str = path.to_string_lossy();
            if path_str.starts_with(r"\\?\") {
                PathBuf::from(&path_str[4..])
            } else {
                path
            }
        }

        #[cfg(not(windows))]
        {
            // On non-Windows systems, just return the path as-is
            path
        }
    }
}

impl std::fmt::Debug for dyn Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Backend({})", self.class_name())
    }
}

impl Default for Box<dyn Backend> {
    fn default() -> Self {
        Box::new(NoneBackend::new())
    }
}

impl dyn Backend {
    pub fn new(filename: &Path) -> Box<dyn Backend> {
        let ext = filename
            .extension()
            .map(|ext| ext.to_str().unwrap_or_default());

        match ext {
            Some("zip") => Box::new(ZipArchive::new(filename)),
            Some("rar") => Box::new(RarArchive::new(filename)),
            Some("mar") => Box::new(MarArchive::new(filename)),
            Some("pdf") | Some("epub") => Box::new(Document::new(filename)),
            Some(_) | None => Box::new(FileSystem::new(filename)),
        }
    }

    pub fn bookmarks(parent_backend: Box<dyn Backend>, parent_target: Target) -> Box<dyn Backend> {
        Box::new(Bookmarks::new(parent_backend, parent_target))
    }

    pub fn thumbnail(thumbnail: Thumbnail) -> Box<dyn Backend> {
        Box::new(thumbnail)
    }

    pub fn none() -> Box<dyn Backend> {
        Box::new(NoneBackend::new())
    }

    pub fn current_dir() -> Box<dyn Backend> {
        match env::current_dir() {
            Ok(cwd) => Box::new(FileSystem::new(&cwd)),
            Err(_) => Box::new(FileSystem::new(&PathBuf::new())),
        }
    }
}
