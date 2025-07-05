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
use cairo::ImageSurface;
use gtk4::ListStore;
use image::{DynamicImage, ImageBuffer, Rgb};
use mupdf::{Colorspace, Device, Matrix, Page, Pixmap, Rect};
use std::path::{Path, PathBuf};

use crate::{
    category::Category,
    error::{MviewError, MviewResult},
    file_view::{Column, Cursor},
    image::{draw::draw_error, provider::gdk::GdkImageLoader, view::Zoom},
    profile::performance::Performance,
};

use super::{
    thumbnail::{TEntry, TReference},
    Backend,
};

const MIN_DOC_HEIGHT: f32 = 32.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PageMode {
    Single,
    #[default]
    DualEvenOdd, // 1, 2-3, 4-5, ...
    DualOddEven, // 1-2, 3-4, 5-6, ...
}

impl From<&str> for PageMode {
    fn from(value: &str) -> Self {
        match value {
            "deo" => PageMode::DualEvenOdd,
            "doe" => PageMode::DualOddEven,
            _ => PageMode::Single,
        }
    }
}

impl From<PageMode> for &str {
    fn from(value: PageMode) -> Self {
        match value {
            PageMode::Single => "single",
            PageMode::DualEvenOdd => "deo",
            PageMode::DualOddEven => "doe",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Pages {
    Single(i32),
    Dual(i32),
}

pub struct Document {
    filename: PathBuf,
    store: ListStore,
    last_page: i32,
}

impl Document {
    pub fn new(filename: &Path) -> Self {
        let (store, last_page) = Self::create_store(filename);
        Document {
            filename: filename.into(),
            store,
            last_page,
        }
    }

    fn create_store(filename: &Path) -> (ListStore, i32) {
        let store = Column::empty_store();
        match list_pages(filename, &store) {
            Ok(last_page) => (store, last_page),
            Err(e) => {
                println!("ERROR {e:?}");
                (store, 0)
            }
        }
    }

    pub fn get_thumbnail(src: &TDocReference) -> MviewResult<DynamicImage> {
        let image = extract_thumb(&src.filename, src.index as i32)?;
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

    fn path(&self) -> PathBuf {
        self.filename.clone()
    }

    fn store(&self) -> ListStore {
        self.store.clone()
    }

    fn image(&self, cursor: &Cursor, params: &ImageParams) -> Image {
        match extract_page(
            &self.filename,
            cursor.index() as i32,
            self.last_page,
            params.page_mode,
            params.allocation_height,
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

    fn image_zoom(
        &self,
        cursor: &Cursor,
        params: &ImageParams,
        current_height: f32,
        clip: Rect,
        zoom: Zoom,
    ) -> Option<ImageSurface> {
        extract_clip(
            &self.filename,
            cursor.index() as i32,
            self.last_page,
            params.page_mode,
            current_height,
            clip,
            zoom.zoom_factor() as f32,
        )
        .ok()
    }
}

//   Single(len=4)  DualOdd(len=6)   DualEven(len=7)
//         0              0                0 1
//         1             1 2               2 3
//         2             3 4               4 5
//         3              5                 6

fn pages(index: i32, last_page: i32, mode: &PageMode) -> Pages {
    match mode {
        PageMode::Single => Pages::Single(index),
        PageMode::DualEvenOdd => {
            if index == 0 {
                Pages::Single(index)
            } else {
                let left = (index - 1) & !1 | 1;
                if left == last_page {
                    Pages::Single(left)
                } else {
                    Pages::Dual(left)
                }
            }
        }
        PageMode::DualOddEven => {
            let left = index & !1;
            if left == last_page {
                Pages::Single(left)
            } else {
                Pages::Dual(left)
            }
        }
    }
}

fn extract_page(
    filename: &Path,
    index: i32,
    last_page: i32,
    mode: &PageMode,
    allocation_height: i32,
) -> MviewResult<Image> {
    match pages(index, last_page, mode) {
        Pages::Single(page) => extract_page_single(filename, page, allocation_height),
        Pages::Dual(left) => extract_page_dual(filename, left, allocation_height),
    }
}

fn extract_page_single(filename: &Path, index: i32, allocation_height: i32) -> MviewResult<Image> {
    let duration = Performance::start();
    let doc = open(filename)?;

    let (page, bounds) = open_page(&doc, index)?;
    let zoom = allocation_height as f32 / bounds.height();
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;

    let image = Image::new_surface(
        GdkImageLoader::surface_from_rgb(pixmap.width(), pixmap.height(), pixmap.samples())?,
        None,
    );
    duration.elapsed("single page");
    Ok(image)
}

fn extract_page_dual(filename: &Path, index: i32, allocation_height: i32) -> MviewResult<Image> {
    let duration = Performance::start();
    let doc = open(filename)?;

    let (page_left, bounds_left) = open_page(&doc, index)?;
    let zoom_left = allocation_height as f32 / bounds_left.height();
    let matrix_left = Matrix::new_scale(zoom_left, zoom_left);
    let pixmap_left = page_left.to_pixmap(&matrix_left, &Colorspace::device_rgb(), false, false)?;

    let (page_right, bounds_right) = open_page(&doc, index + 1)?;
    let zoom_right = allocation_height as f32 / bounds_right.height();
    let matrix_right = Matrix::new_scale(zoom_right, zoom_right);
    let pixmap_right =
        page_right.to_pixmap(&matrix_right, &Colorspace::device_rgb(), false, false)?;

    let image = Image::new_dual_surface(
        GdkImageLoader::surface_from_rgb(
            pixmap_left.width(),
            pixmap_left.height(),
            pixmap_left.samples(),
        )
        .ok(),
        GdkImageLoader::surface_from_rgb(
            pixmap_right.width(),
            pixmap_right.height(),
            pixmap_right.samples(),
        )
        .ok(),
        None,
    );
    duration.elapsed("dual page");
    Ok(image)
}

fn extract_thumb(filename: &Path, index: i32) -> MviewResult<DynamicImage> {
    let doc = open(filename)?;

    let (page, bounds) = open_page(&doc, index)?;
    let zoom = 350.0 / bounds.height();
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;

    match ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        pixmap.width(),
        pixmap.height(),
        pixmap.samples().to_vec(),
    ) {
        Some(rgb_image) => Ok(DynamicImage::ImageRgb8(rgb_image)),
        None => Err("Could not create ImageBuffer from pdf thumb data".into()),
    }
}

fn extract_clip(
    filename: &Path,
    index: i32,
    last_page: i32,
    mode: &PageMode,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<ImageSurface> {
    match pages(index, last_page, mode) {
        Pages::Single(page) => extract_clip_single(filename, page, current_height, clip, zoom),
        Pages::Dual(left) => extract_clip_dual(filename, left, current_height, clip, zoom),
    }
}

fn extract_clip_single(
    filename: &Path,
    index: i32,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<ImageSurface> {
    let duration = Performance::start();
    let doc = open(filename)?;

    let surface = if let Some(pixmap) = doc_extract_clip(&doc, index, current_height, clip, zoom)? {
        Ok(GdkImageLoader::surface_from_rgb(
            pixmap.width(),
            pixmap.height(),
            pixmap.samples(),
        )?)
    } else {
        Err("empty clip".into())
    };

    duration.elapsed("clipped page");
    surface
}

fn extract_clip_dual(
    filename: &Path,
    index: i32,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<ImageSurface> {
    let duration = Performance::start();
    let doc = open(filename)?;

    let (page_left, bounds_left) = open_page(&doc, index)?;
    let offset_right = bounds_left.width() * current_height / bounds_left.height();
    let clip_right = clip.translate(-offset_right, 0.0);

    let pixmap_left = page_extract_clip(&page_left, bounds_left, current_height, clip, zoom)?;
    let pixmap_right = doc_extract_clip(&doc, index + 1, current_height, clip_right, zoom)?;

    let surface = match (pixmap_left, pixmap_right) {
        (None, None) => return Err("empty clip".into()),
        (Some(pixmap_left), None) => GdkImageLoader::surface_from_rgb(
            pixmap_left.width(),
            pixmap_left.height(),
            pixmap_left.samples(),
        )?,
        (None, Some(pixmap_right)) => GdkImageLoader::surface_from_rgb(
            pixmap_right.width(),
            pixmap_right.height(),
            pixmap_right.samples(),
        )?,
        (Some(pixmap_left), Some(pixmap_right)) => {
            if pixmap_left.height() != pixmap_right.height() {
                return Err("height mismatch".into());
            }
            GdkImageLoader::surface_from_dual_rgb(
                pixmap_left.width(),
                pixmap_right.width(),
                pixmap_left.height(),
                pixmap_left.samples(),
                pixmap_right.samples(),
            )?
        }
    };

    duration.elapsed("clipped dual");
    Ok(surface)
}

fn open_page(doc: &mupdf::Document, page_no: i32) -> MviewResult<(Page, Rect)> {
    let page = doc.load_page(page_no)?;
    let bounds = page.bounds()?;
    if bounds.height() < MIN_DOC_HEIGHT {
        return Err("page height too small".into());
    }
    Ok((page, bounds))
}

fn doc_extract_clip(
    doc: &mupdf::Document,
    page_no: i32,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<Option<mupdf::Pixmap>> {
    let (page, page_bounds) = open_page(doc, page_no)?;
    page_extract_clip(&page, page_bounds, current_height, clip, zoom)
}

fn page_extract_clip(
    page: &Page,
    page_bounds: Rect,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<Option<mupdf::Pixmap>> {
    // use `current_height` to determine `current_zoom`
    let current_zoom = current_height / page_bounds.height();
    if current_zoom < 1e-3 {
        return Err("current_zoom value out of range".into());
    }

    // Clip is zoomed by `current_zoom`: unzoom
    let matrix = Matrix::new_scale(1.0 / current_zoom, 1.0 / current_zoom);
    let clip = clip.transform(&matrix);

    // Determine intersection between `clip`` and `page_bounds`
    let intersect = page_bounds.intersect(&clip);

    // New zoom is `zoom` * `current_zoom`
    let matrix = Matrix::new_scale(zoom * current_zoom, zoom * current_zoom);
    let intersect = intersect.transform(&matrix).round();

    if intersect.is_empty() {
        Ok(None) // clip intersection is empty
    } else {
        let mut pixmap = Pixmap::new_with_rect(&Colorspace::device_rgb(), intersect, false)?;
        pixmap.clear_with(0xff)?;

        let device = Device::from_pixmap(&pixmap)?;
        page.run_contents(&device, &matrix)?;
        Ok(Some(pixmap))
    }
}

fn open(path: &Path) -> Result<mupdf::Document, mupdf::Error> {
    #[cfg(windows)]
    {
        mupdf::Document::open(&path.to_string_lossy().to_string())
    }

    #[cfg(not(windows))]
    {
        mupdf::Document::open(path)
    }
}

fn list_pages(filename: &Path, store: &ListStore) -> MviewResult<i32> {
    let duration = Performance::start();
    let doc = open(filename)?;
    let page_count = doc.page_count()? as u32;
    println!("Total pages: {page_count}");
    if page_count > 0 {
        let cat = Category::Image;
        for i in 0..page_count {
            let page = format!("Page {0:5}", i + 1);
            store.insert_with_values(
                None,
                &[
                    (Column::Cat as u32, &cat.id()),
                    (Column::Icon as u32, &cat.icon()),
                    (Column::Name as u32, &page),
                    (Column::Index as u32, &i),
                ],
            );
        }
        duration.elapsed("list_pages");
        Ok(page_count as i32 - 1)
    } else {
        Err(MviewError::from("No pages in document"))
    }
}

#[derive(Debug, Clone)]
pub struct TDocReference {
    filename: PathBuf,
    index: u64,
}

impl TDocReference {
    pub fn new(backend: &Document, index: u64) -> Self {
        TDocReference {
            filename: backend.filename.clone(),
            index,
        }
    }

    pub fn filename(&self) -> PathBuf {
        self.filename.clone()
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}
