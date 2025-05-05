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
use gtk4::ListStore;
use image::{DynamicImage, ImageBuffer, Rgb};
use mupdf::{Colorspace, Matrix};
use std::{
    cell::{Cell, RefCell},
    path::Path,
};

use crate::{
    category::Category,
    error::{MviewError, MviewResult},
    file_view::{Columns, Cursor, Sort},
    image::{draw::draw_error, provider::gdk::GdkImageLoader},
    profile::performance::Performance,
};

use super::{
    filesystem::FileSystem,
    thumbnail::{TEntry, TReference},
    Backend, Selection,
};

#[derive(Clone, Copy, Debug, Default)]
pub enum PageMode {
    #[default]
    Single,
    DualOdd,
    DualEven,
}

pub struct Document {
    filename: String,
    directory: String,
    archive: String,
    store: ListStore,
    last_page: u32,
    parent: RefCell<Box<dyn Backend>>,
    sort: Cell<Sort>,
}

impl Document {
    pub fn new(filename: &str) -> Self {
        let path = Path::new(filename);
        let directory = path
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_str()
            .unwrap_or("/");
        let archive = path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        let (store, last_page) = Self::create_store(filename);
        Document {
            filename: filename.to_string(),
            directory: directory.to_string(),
            archive: archive.to_string(),
            store,
            last_page,
            parent: RefCell::new(<dyn Backend>::none()),
            sort: Default::default(),
        }
    }

    fn create_store(filename: &str) -> (ListStore, u32) {
        let store = Columns::store();
        match list_pages(filename, &store) {
            Ok(last_page) => (store, last_page),
            Err(e) => {
                println!("ERROR {:?}", e);
                (store, 0)
            }
        }
    }

    pub fn get_thumbnail(src: &TDocReference) -> MviewResult<DynamicImage> {
        let image = extract_page_thumb(&src.filename, src.index as i32)?;
        let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
        Ok(image)
    }
}

impl Backend for Document {
    fn class_name(&self) -> &str {
        "Document"
    }

    fn is_container(&self) -> bool {
        true
    }

    fn is_doc(&self) -> bool {
        true
    }

    fn path(&self) -> &str {
        &self.filename
    }

    fn store(&self) -> ListStore {
        self.store.clone()
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        if self.parent.borrow().is_none() {
            (
                Box::new(FileSystem::new(&self.directory)),
                Selection::Name(self.archive.clone()),
            )
        } else {
            (
                self.parent.replace(<dyn Backend>::none()),
                Selection::Name(self.archive.clone()),
            )
        }
    }

    fn image(&self, cursor: &Cursor, params: &ImageParams) -> Image {
        match extract_page(
            &self.filename,
            cursor.index(),
            self.last_page,
            params.page_mode,
        ) {
            Ok(image) => image,
            Err(error) => draw_error(error.to_string().into()),
        }
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        TEntry::new(
            cursor.category(),
            &cursor.name(),
            TReference::DocReference(TDocReference::new(self, cursor.index())),
        )
    }

    fn set_parent(&self, parent: Box<dyn Backend>) {
        if self.parent.borrow().is_none() {
            self.parent.replace(parent);
        }
    }

    fn set_sort(&self, sort: &Sort) {
        self.sort.set(*sort)
    }

    fn sort(&self) -> Sort {
        self.sort.get()
    }
}

//   Single(len=4)  DualOdd(len=6)   DualEven(len=7)
//         0              0                0 1
//         1             1 2               2 3
//         2             3 4               4 5
//         3              5                 6

fn extract_page(
    filename: &str,
    index: u32,
    last_page: u32,
    mode: &PageMode,
) -> Result<Image, mupdf::Error> {
    match mode {
        PageMode::Single => extract_page_single(filename, index),
        PageMode::DualOdd => {
            if index == 0 {
                extract_page_single(filename, index)
            } else {
                let left = (index - 1) & !1 | 1;
                if left == last_page {
                    extract_page_single(filename, left)
                } else {
                    extract_page_dual(filename, left)
                }
            }
        }
        PageMode::DualEven => {
            let left = index & !1;
            if left == last_page {
                extract_page_single(filename, left)
            } else {
                extract_page_dual(filename, left)
            }
        }
    }
}

fn extract_page_single(filename: &str, index: u32) -> Result<Image, mupdf::Error> {
    let doc = mupdf::Document::open(filename)?;
    let page = doc.load_page(index as i32)?;
    let bounds = page.bounds()?;
    let height = bounds.y1 - bounds.y0;
    let zoom = if height > 10.0 { 2160.0 / height } else { 3.0 };
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;
    Ok(Image::new_pixbuf(
        Some(GdkImageLoader::pixbuf_from_rgb(
            pixmap.width(),
            pixmap.height(),
            pixmap.samples(),
        )),
        None,
    ))
}

fn extract_page_dual(filename: &str, index: u32) -> Result<Image, mupdf::Error> {
    let doc = mupdf::Document::open(filename)?;

    let page = doc.load_page(index as i32)?;
    let bounds = page.bounds()?;
    let height = bounds.y1 - bounds.y0;
    let zoom = if height > 10.0 { 2160.0 / height } else { 3.0 };
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap1 = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;

    let page = doc.load_page(index as i32 + 1)?;
    let bounds = page.bounds()?;
    let height = bounds.y1 - bounds.y0;
    let zoom = if height > 10.0 { 2160.0 / height } else { 3.0 };
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap2 = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;

    Ok(Image::new_dual_pixbuf(
        Some(GdkImageLoader::pixbuf_from_rgb(
            pixmap1.width(),
            pixmap1.height(),
            pixmap1.samples(),
        )),
        Some(GdkImageLoader::pixbuf_from_rgb(
            pixmap2.width(),
            pixmap2.height(),
            pixmap2.samples(),
        )),
        None,
    ))
}

fn extract_page_thumb(filename: &str, index: i32) -> MviewResult<DynamicImage> {
    let doc = mupdf::Document::open(filename)?;
    let page = doc.load_page(index)?;
    let bounds = page.bounds()?;
    let height = bounds.y1 - bounds.y0;
    let zoom = if height > 10.0 { 350.0 / height } else { 1.0 };
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;

    match ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        pixmap.width(),
        pixmap.height(),
        pixmap.samples().to_vec(),
    ) {
        Some(rgb_image) => Ok(DynamicImage::ImageRgb8(rgb_image)),
        None => Err(MviewError::from(
            "Could not create ImageBuffer from pdf thumb data",
        )),
    }
}

fn list_pages(filename: &str, store: &ListStore) -> MviewResult<u32> {
    let duration = Performance::start();
    let doc = mupdf::Document::open(filename)?;
    let page_count = doc.page_count()? as u32;
    println!("Total pages: {}", page_count);
    if page_count > 0 {
        let cat = Category::Image;
        for i in 0..page_count {
            let page = format!("Page {0:5}", i + 1);
            store.insert_with_values(
                None,
                &[
                    (Columns::Cat as u32, &cat.id()),
                    (Columns::Icon as u32, &cat.icon()),
                    (Columns::Name as u32, &page),
                    (Columns::Index as u32, &i),
                ],
            );
        }
        duration.elapsed("list_pages");
        Ok(page_count - 1)
    } else {
        Err(MviewError::from("No pages in document"))
    }
}

#[derive(Debug, Clone)]
pub struct TDocReference {
    filename: String,
    index: u32,
}

impl TDocReference {
    pub fn new(backend: &Document, index: u32) -> Self {
        TDocReference {
            filename: backend.filename.clone(),
            index,
        }
    }

    pub fn filename(&self) -> String {
        self.filename.clone()
    }

    pub fn index(&self) -> u32 {
        self.index
    }
}
