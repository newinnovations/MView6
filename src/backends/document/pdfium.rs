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

use cairo::ImageSurface;
use gtk4::ListStore;
use mupdf::{Matrix, Rect};
use pdfium::{PdfiumBitmap, PdfiumDocument, PdfiumPage, PdfiumRenderConfig};
use std::path::{Path, PathBuf};

use crate::{
    backends::{
        document::{mupdf::TDocReference, pages, PageMode, Pages, MIN_DOC_HEIGHT},
        thumbnail::{TEntry, TReference},
        Backend, ImageParams,
    },
    category::Category,
    error::MviewResult,
    file_view::{Column, Cursor},
    image::{draw::draw_error, provider::surface::Surface, view::Zoom, Image},
    profile::performance::Performance,
};

pub struct DocPdfium {
    filename: PathBuf,
    document: MviewResult<PdfiumDocument>,
    store: ListStore,
    last_page: i32,
}

impl DocPdfium {
    pub fn new(filename: &Path) -> Self {
        let (store, document, last_page) = Self::create_store(filename);
        DocPdfium {
            filename: filename.into(),
            document,
            store,
            last_page,
        }
    }

    fn create_store(filename: &Path) -> (ListStore, MviewResult<PdfiumDocument>, i32) {
        let store = Column::empty_store();
        match list_pages(filename, &store) {
            Ok((document, last_page)) => (store, Ok(document), last_page),
            Err(e) => {
                println!("ERROR {e:?}");
                (store, Err(e), 0)
            }
        }
    }
}

impl Backend for DocPdfium {
    fn class_name(&self) -> &str {
        "DocPdfium"
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
        (|| {
            let document = self.document.as_ref().map_err(|e| e.to_string())?;
            extract_page(
                document,
                cursor.index() as i32,
                self.last_page,
                params.page_mode,
                params.allocation_height,
            )
            .map_err(|e| e.to_string())
        })()
        .unwrap_or_else(|e| draw_error(e.into()))
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        TEntry::new(
            cursor.category(),
            &cursor.name(),
            TReference::DocReference(TDocReference {
                filename: self.filename.clone(),
                index: cursor.index(),
            }),
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
        let document = self.document.as_ref().ok()?;
        extract_clip(
            document,
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

fn extract_page(
    document: &PdfiumDocument,
    index: i32,
    last_page: i32,
    mode: &PageMode,
    allocation_height: i32,
) -> MviewResult<Image> {
    match pages(index, last_page, mode) {
        Pages::Single(page) => extract_page_single(document, page, allocation_height),
        Pages::Dual(left) => extract_page_dual(document, left, allocation_height),
    }
}

fn extract_page_single(document: &PdfiumDocument, index: i32, height: i32) -> MviewResult<Image> {
    let duration = Performance::start();
    let surface = page_to_surface(document, index, height)?;
    let result = Ok(Image::new_surface(surface, None));
    duration.elapsed("pdfium single");
    result
}

fn extract_page_dual(document: &PdfiumDocument, index: i32, height: i32) -> MviewResult<Image> {
    let duration = Performance::start();
    let surface_left = page_to_surface(document, index, height)?;
    let surface_right = page_to_surface(document, index + 1, height)?;
    let result = Ok(Image::new_dual_surface(
        Some(surface_left),
        Some(surface_right),
        None,
    ));
    duration.elapsed("pdfium dual");
    result
}

fn page_to_surface(
    document: &PdfiumDocument,
    index: i32,
    height: i32,
) -> MviewResult<ImageSurface> {
    let page = document.page(index)?;
    let bounds = page.boundaries().media()?;
    if bounds.height() < MIN_DOC_HEIGHT {
        return Err("page height too small".into());
    }
    let zoom = height as f32 / bounds.height();
    let width = (bounds.width() * zoom) as i32;
    let config = PdfiumRenderConfig::new()
        .with_size(width, height)
        .with_scale(zoom);
    let bitmap = page.render(&config)?;
    Surface::from_bgra8_bytes(width as u32, height as u32, bitmap.as_raw_bytes())
}

fn extract_clip(
    document: &PdfiumDocument,
    index: i32,
    last_page: i32,
    mode: &PageMode,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<ImageSurface> {
    match pages(index, last_page, mode) {
        Pages::Single(page) => extract_clip_single(document, page, current_height, clip, zoom),
        Pages::Dual(left) => extract_clip_dual(document, left, current_height, clip, zoom),
    }
}

fn extract_clip_single(
    document: &PdfiumDocument,
    index: i32,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<ImageSurface> {
    let duration = Performance::start();
    let page = document.page(index)?;
    let surface = if let Some(bitmap) = page_extract_clip(&page, current_height, clip, zoom)? {
        Ok(Surface::from_bgra8_bytes(
            bitmap.width() as u32,
            bitmap.height() as u32,
            bitmap.as_raw_bytes(),
        )?)
    } else {
        Err("empty clip".into())
    };
    duration.elapsed("pdfium clip:1");
    surface
}

fn extract_clip_dual(
    document: &PdfiumDocument,
    index: i32,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<ImageSurface> {
    let duration = Performance::start();

    // let (page_left, bounds_left) = open_page(&document, index)?;
    let page_left = document.page(index)?;
    let bounds_left = page_left.boundaries().media()?;
    if bounds_left.height() < MIN_DOC_HEIGHT {
        return Err("page height too small".into());
    }

    let offset_right = bounds_left.width() * current_height / bounds_left.height();
    let clip_right = clip.translate(-offset_right, 0.0);

    let pixmap_left = page_extract_clip(&page_left, current_height, clip, zoom)?;

    let page_right = document.page(index + 1)?;
    let pixmap_right = page_extract_clip(&page_right, current_height, clip_right, zoom)?;

    let surface = match (pixmap_left, pixmap_right) {
        (None, None) => return Err("empty clip".into()),
        (Some(pixmap_left), None) => Surface::from_bgra8_bytes(
            pixmap_left.width() as u32,
            pixmap_left.height() as u32,
            pixmap_left.as_raw_bytes(),
        )?,
        (None, Some(pixmap_right)) => Surface::from_bgra8_bytes(
            pixmap_right.width() as u32,
            pixmap_right.height() as u32,
            pixmap_right.as_raw_bytes(),
        )?,
        (Some(pixmap_left), Some(pixmap_right)) => {
            if pixmap_left.height() != pixmap_right.height() {
                return Err("height mismatch".into());
            }
            Surface::from_dual_bgra8_bytes(
                pixmap_left.width() as u32,
                pixmap_left.height() as u32,
                pixmap_left.as_raw_bytes(),
                pixmap_right.width() as u32,
                pixmap_right.height() as u32,
                pixmap_right.as_raw_bytes(),
            )?
        }
    };

    duration.elapsed("pdfium clip:2");
    Ok(surface)
}

fn page_extract_clip(
    page: &PdfiumPage,
    current_height: f32,
    clip: Rect,
    zoom: f32,
) -> MviewResult<Option<PdfiumBitmap>> {
    let bounds = page.boundaries().media()?;
    if bounds.height() < MIN_DOC_HEIGHT {
        return Err("page height too small".into());
    }

    // use `current_height` to determine `current_zoom`
    let current_zoom = current_height / bounds.height();
    if current_zoom < 1e-3 {
        return Err("current_zoom value out of range".into());
    }

    // Clip is zoomed by `current_zoom`: unzoom
    let matrix = Matrix::new_scale(1.0 / current_zoom, 1.0 / current_zoom);
    let clip = clip.transform(&matrix);

    // For the intersect algortihm to work we need to
    // - convert the pdfium::PdfRect to mupdf::Rect
    // - set the origin at (0,0) which is not always the case with pdfium
    let page_bounds = Rect::new(0.0, 0.0, bounds.width(), bounds.height());

    // Determine intersection between `clip`` and `page_bounds`
    let intersect = page_bounds.intersect(&clip);

    // New zoom is `zoom` * `current_zoom`
    let new_zoom = zoom * current_zoom;
    let matrix = Matrix::new_scale(new_zoom, new_zoom);
    let intersect = intersect.transform(&matrix).round();

    if intersect.is_empty() {
        Ok(None) // clip intersection is empty
    } else {
        let config = PdfiumRenderConfig::new()
            .with_size(intersect.width(), intersect.height())
            .with_scale(new_zoom)
            .with_pan(-intersect.x0 as f32, -intersect.y0 as f32);
        Ok(Some(page.render(&config)?))
    }
}

fn list_pages(filename: &Path, store: &ListStore) -> MviewResult<(PdfiumDocument, i32)> {
    let duration = Performance::start();
    let document = PdfiumDocument::new_from_path(filename, None)?;
    let page_count = document.page_count();
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
        duration.elapsed("pdfium list");
        Ok((document, page_count - 1))
    } else {
        Err("No pages in document".into())
    }
}
