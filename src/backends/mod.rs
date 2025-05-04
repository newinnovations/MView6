// MView6 -- Opiniated image browser written in Rust and GTK4
//
// Copyright (c) 2024 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU General Public License as published by the Free Software Foundation, either version 3
// of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::env;

use archive_rar::RarArchive;
use archive_zip::ZipArchive;
use async_channel::Sender;
use bookmarks::Bookmarks;
use filesystem::FileSystem;
use gtk4::ListStore;
use none::NoneBackend;
use pdf::{Pdf, PdfMode};
use thumbnail::{Message, TEntry, Thumbnail};

use crate::{
    file_view::{Cursor, Direction, Selection, Sort},
    image::Image,
};

mod archive_rar;
mod archive_zip;
mod bookmarks;
pub mod filesystem;
mod none;
pub mod pdf;
pub mod thumbnail;

pub struct ImageParams<'a> {
    pub sender: &'a Sender<Message>,
    pub pdf_mode: &'a PdfMode,
}

#[allow(unused_variables)]
pub trait Backend {
    fn class_name(&self) -> &str;
    fn path(&self) -> &str;
    fn store(&self) -> ListStore;
    fn favorite(&self, cursor: &Cursor, direction: Direction) -> bool {
        false
    }
    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        None
    }
    fn leave(&self) -> (Box<dyn Backend>, Selection);
    // fn image(&self, cursor: &Cursor, sender: &Sender<Message>, pdf_mode: &PdfMode) -> Image;
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
    fn is_none(&self) -> bool {
        false
    }
    fn click(&self, current: &Cursor, x: f64, y: f64) -> Option<(Box<dyn Backend>, Selection)> {
        None
    }
    fn set_parent(&self, parent: Box<dyn Backend>) {}
    fn set_sort(&self, sort: &Sort);
    fn sort(&self) -> Sort;
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
    pub fn new(filename: &str) -> Box<dyn Backend> {
        if filename.ends_with(".zip") {
            Box::new(ZipArchive::new(filename))
        } else if filename.ends_with(".rar") {
            Box::new(RarArchive::new(filename))
        } else if filename.ends_with(".pdf") {
            Box::new(Pdf::new(filename))
        } else {
            Box::new(FileSystem::new(filename))
        }
    }

    pub fn bookmarks() -> Box<dyn Backend> {
        Box::new(Bookmarks::new())
    }

    pub fn thumbnail(thumbnail: Thumbnail) -> Box<dyn Backend> {
        Box::new(thumbnail)
    }

    pub fn none() -> Box<dyn Backend> {
        Box::new(NoneBackend::new())
    }

    pub fn current_dir() -> Box<dyn Backend> {
        match env::current_dir() {
            Ok(cwd) => Box::new(FileSystem::new(cwd.as_os_str().to_str().unwrap_or("/"))),
            Err(_) => Box::new(FileSystem::new("/")),
        }
    }
}
