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

use std::{
    env,
    path::{Path, PathBuf},
};

use async_channel::Sender;

#[cfg(feature = "mupdf")]
use crate::backends::document::{mupdf::DocMuPdf, PdfEngine};

use crate::{
    backends::{
        document::{pdf_engine, pdfium::DocPdfium, PageMode},
        thumbnail::model::TParent,
        Bookmarks, FileSystem, MarArchive, Message, NoneBackend, RarArchive, Thumbnail, ZipArchive,
    },
    content::Content,
    error::MviewResult,
    file_view::{BackendRef, Column, Cursor, Direction, ItemRef, Reference, Row, Target},
    image::{SurfaceData, Zoom},
    rect::{PointD, RectD},
    util::path_to_filename,
};

pub struct ImageParams<'a> {
    pub tn_sender: Option<&'a Sender<Message>>,
    pub page_mode: &'a PageMode,
    pub allocation_height: i32,
}

#[allow(unused_variables)]
pub trait Backend {
    fn class_name(&self) -> &str;
    fn path(&self) -> PathBuf;
    fn list(&self) -> &[Row];
    fn set_preference(&self, cursor: &Cursor, direction: Direction) -> bool {
        false
    }
    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        if let Some(parent) = self.path().parent() {
            match FileSystem::try_new(parent) {
                Ok(new_backend) => Some((
                    Box::new(new_backend),
                    Target::Name(path_to_filename(self.path())),
                )),
                Err(e) => {
                    eprintln!("Failed to leave directory: {e}");
                    None
                }
            }
        } else {
            None
        }
    }

    fn backend_ref(&self) -> BackendRef;
    fn item_ref(&self, cursor: &Cursor) -> ItemRef;

    fn enter(&self, cursor: &Cursor) -> Option<Box<dyn Backend>> {
        None
    }

    fn content(&self, item: &ItemRef, params: &ImageParams) -> Content;
    fn click(&self, item: &ItemRef, mouse_pos: PointD) -> Option<(Box<dyn Backend>, Target)> {
        None
    }

    fn render(
        &self,
        item: &ItemRef,
        page_mode: &PageMode,
        zoom: &Zoom,
        viewport: &RectD,
    ) -> Option<SurfaceData> {
        None
    }

    // Only implemented by thumbnail backend, dummy here
    fn get_thumb_parent(&self) -> TParent {
        TParent {
            backend: <dyn Backend>::none(),
            target: Target::First,
            focus_pos: 0,
            store: Column::empty_store(),
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
    pub fn new_from_path(filename: &Path) -> MviewResult<Box<dyn Backend>> {
        let ext = filename
            .extension()
            .map(|ext| ext.to_string_lossy().to_ascii_lowercase());

        Ok(match ext.as_deref() {
            Some("zip") => Box::new(ZipArchive::try_new(filename)?),
            Some("rar") => Box::new(RarArchive::try_new(filename)?),
            Some("mar") => Box::new(MarArchive::try_new(filename)?),
            Some("pdf") => match pdf_engine() {
                #[cfg(feature = "mupdf")]
                PdfEngine::MuPdf => Box::new(DocMuPdf::try_new(filename)?),
                _ => Box::new(DocPdfium::try_new(filename)?),
            },
            #[cfg(feature = "mupdf")]
            Some("epub") => Box::new(DocMuPdf::try_new(filename)?),
            Some(_) | None => Box::new(FileSystem::try_new(filename)?),
        })
    }

    pub fn new_from_ref(reference: &BackendRef) -> MviewResult<Box<dyn Backend>> {
        Ok(match reference {
            BackendRef::FileSystem(path_buf) => Box::new(FileSystem::try_new(path_buf)?),
            BackendRef::MarArchive(path_buf) => Box::new(MarArchive::try_new(path_buf)?),
            BackendRef::RarArchive(path_buf) => Box::new(RarArchive::try_new(path_buf)?),
            BackendRef::ZipArchive(path_buf) => Box::new(ZipArchive::try_new(path_buf)?),
            #[cfg(feature = "mupdf")]
            BackendRef::Mupdf(path_buf) => Box::new(DocMuPdf::try_new(path_buf)?),
            BackendRef::Pdfium(path_buf) => Box::new(DocPdfium::try_new(path_buf)?),
            // BackendRef::Thumbnail => Box::new(todo!()),
            // BackendRef::Bookmarks => Box::new(todo!()),
            // BackendRef::None => Box::new(todo!()),
            _ => Box::new(NoneBackend::new()),
        })
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
            Ok(cwd) => match FileSystem::try_new(&cwd) {
                Ok(new_backend) => Box::new(new_backend),
                Err(e) => {
                    eprintln!("Failed to initialize filesystem backend for cwd {cwd:?}: {e}");
                    Box::new(NoneBackend::new())
                }
            },
            Err(_) => {
                eprintln!("Failed to get current directory");
                Box::new(NoneBackend::new())
            }
        }
    }

    pub fn reference(&self, cursor: &Cursor) -> Reference {
        Reference {
            backend: self.backend_ref(),
            item: self.item_ref(cursor),
        }
    }

    pub fn can_show_thumbnails(&self) -> bool {
        !matches!(
            self.backend_ref(),
            BackendRef::Thumbnail | BackendRef::Bookmarks | BackendRef::None
        )
    }

    pub fn is_filesystem(&self) -> bool {
        matches!(self.backend_ref(), BackendRef::FileSystem(_))
    }

    pub fn is_bookmarks(&self) -> bool {
        matches!(self.backend_ref(), BackendRef::Bookmarks)
    }

    pub fn is_thumbnail(&self) -> bool {
        matches!(self.backend_ref(), BackendRef::Thumbnail)
    }

    pub fn is_doc(&self) -> bool {
        matches!(
            self.backend_ref(),
            BackendRef::Pdfium(_) | BackendRef::Mupdf(_)
        )
    }

    pub fn is_none(&self) -> bool {
        matches!(self.backend_ref(), BackendRef::None)
    }

    pub fn can_be_sorted(&self) -> bool {
        !matches!(
            self.backend_ref(),
            BackendRef::Pdfium(_) | BackendRef::Mupdf(_) | BackendRef::Thumbnail
        )
    }

    pub fn supports_filter(&self) -> bool {
        !matches!(
            self.backend_ref(),
            BackendRef::Pdfium(_)
                | BackendRef::Mupdf(_)
                | BackendRef::Thumbnail
                | BackendRef::Bookmarks
                | BackendRef::None
        )
    }
}
