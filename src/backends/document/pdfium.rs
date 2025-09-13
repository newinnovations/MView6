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

use image::DynamicImage;
use pdfium::{PdfiumBitmap, PdfiumDocument, PdfiumPage, PdfiumRenderConfig};
use std::path::{Path, PathBuf};

use crate::{
    backends::{
        document::{pages, PageMode, Pages},
        Backend, ImageParams,
    },
    category::Category,
    content::Content,
    error::MviewResult,
    file_view::{
        model::{BackendRef, ItemRef, Reference, Row},
        Cursor,
    },
    image::{draw::draw_error, provider::surface::SurfaceData, view::Zoom},
    profile::performance::Performance,
    rect::{RectD, SizeD, VectorD},
};

pub struct DocPdfium {
    path: PathBuf,
    document: MviewResult<PdfiumDocument>,
    store: Vec<Row>,
    last_page: i32,
}

impl DocPdfium {
    pub fn new(filename: &Path) -> Self {
        let (document, store, last_page) = Self::create_store(filename);
        DocPdfium {
            path: filename.into(),
            document,
            store,
            last_page,
        }
    }

    fn create_store(filename: &Path) -> (MviewResult<PdfiumDocument>, Vec<Row>, i32) {
        match list_pages(filename) {
            Ok((document, store, last_page)) => (Ok(document), store, last_page),
            Err(e) => {
                eprintln!("ERROR {e:?}");
                (Err(e), Default::default(), Default::default())
            }
        }
    }

    pub fn get_thumbnail(src: &Reference) -> MviewResult<DynamicImage> {
        if let (BackendRef::Pdfium(filename), ItemRef::Index(index)) = src.as_tuple() {
            let image = extract_thumb(filename, *index as i32)?;
            let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
            Ok(image)
        } else {
            Err("invalid reference".into())
        }
    }
}

impl Backend for DocPdfium {
    fn class_name(&self) -> &str {
        "PDFium"
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn list(&self) -> &Vec<Row> {
        &self.store
    }

    fn content(&self, item: &ItemRef, params: &ImageParams) -> Content {
        (|| {
            let document = self.document.as_ref().map_err(|e| e.to_string())?;
            page_size(
                Reference {
                    backend: BackendRef::Pdfium(self.path.clone()),
                    item: item.clone(),
                },
                document,
                item.idx() as i32,
                self.last_page,
                params.page_mode,
            )
            .map_err(|e| e.to_string())
        })()
        .unwrap_or_else(|e| draw_error(e.into()))
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::Pdfium(self.path.clone())
    }

    fn item_ref(&self, cursor: &Cursor) -> ItemRef {
        ItemRef::Index(cursor.index())
    }

    fn render(
        &self,
        item: &ItemRef,
        page_mode: &PageMode,
        zoom: &Zoom,
        viewport: &RectD,
    ) -> Option<SurfaceData> {
        let document = self.document.as_ref().ok()?;
        render(
            document,
            item.idx() as i32,
            self.last_page,
            page_mode,
            zoom,
            viewport,
        )
        .ok()
    }
}

fn page_size(
    reference: Reference,
    document: &PdfiumDocument,
    index: i32,
    last_page: i32,
    mode: &PageMode,
) -> MviewResult<Content> {
    match pages(index, last_page, mode) {
        Pages::Single(page) => page_size_single(reference, mode, document, page),
        Pages::Dual(left) => page_size_dual(reference, mode, document, left),
    }
}

fn page_size_single(
    reference: Reference,
    mode: &PageMode,
    document: &PdfiumDocument,
    index: i32,
) -> MviewResult<Content> {
    let duration = Performance::start();
    let size = page_size_as_rect(&document.page(index)?)?;
    let image = Content::new_doc(reference, *mode, size);
    duration.elapsed("pdfium single");
    Ok(image)
}

fn page_size_dual(
    reference: Reference,
    mode: &PageMode,
    document: &PdfiumDocument,
    index: i32,
) -> MviewResult<Content> {
    // The right page is scaled so its height is the same as the left page
    let duration = Performance::start();
    let size_left = page_size_as_rect(&document.page(index)?)?;
    let size_right = page_size_as_rect(&document.page(index + 1)?)?;
    let scale_right = size_left.height() / size_right.height();
    let size = SizeD::new(
        size_left.width() + scale_right * size_right.width(),
        size_left.height(),
    );
    let image = Content::new_doc(reference, *mode, size);
    duration.elapsed("pdfium dual");
    Ok(image)
}

fn extract_thumb(filename: &Path, index: i32) -> MviewResult<DynamicImage> {
    let document = PdfiumDocument::new_from_path(filename, None)?;
    let page = document.page(index)?;
    let zoom = 350.0 / page.height();
    let width = (page.width() * zoom) as i32;
    let config = PdfiumRenderConfig::new()
        .with_size(width, 350)
        .with_scale(zoom);
    let bitmap = page.render(&config)?;
    Ok(bitmap.as_rgba8_image()?)
}

fn page_size_as_rect(page: &PdfiumPage) -> MviewResult<SizeD> {
    Ok(SizeD::new(page.width() as f64, page.height() as f64))
}

fn render(
    document: &PdfiumDocument,
    index: i32,
    last_page: i32,
    mode: &PageMode,
    zoom: &Zoom,
    viewport: &RectD,
) -> MviewResult<SurfaceData> {
    match pages(index, last_page, mode) {
        Pages::Single(page) => render_single(document, page, zoom, viewport),
        Pages::Dual(left) => render_dual(document, left, zoom, viewport),
    }
}

fn render_single(
    document: &PdfiumDocument,
    index: i32,
    zoom: &Zoom,
    viewport: &RectD,
) -> MviewResult<SurfaceData> {
    let duration = Performance::start();
    let page = document.page(index)?;
    let surface = if let Some(bitmap) = page_render(&page, zoom, viewport)? {
        Ok(SurfaceData::from_bgra8(
            bitmap.width() as u32,
            bitmap.height() as u32,
            bitmap.as_raw_bytes(),
        ))
    } else {
        Err("empty clip".into())
    };
    duration.elapsed("pdfium clip:1");
    surface
}

fn render_dual(
    document: &PdfiumDocument,
    index: i32,
    zoom: &Zoom,
    viewport: &RectD,
) -> MviewResult<SurfaceData> {
    let duration = Performance::start();

    let page_left = document.page(index)?;
    let size_left = page_size_as_rect(&page_left)?;
    let mut zoom_left = zoom.clone();
    zoom_left.set_image_size(size_left);
    let pixmap_left = page_render(&page_left, &zoom_left, viewport)?;

    let page_right = document.page(index + 1)?;
    let size_right = page_size_as_rect(&page_right)?;
    let scale_right = size_left.height() / size_right.height();
    let mut zoom_right = zoom.clone();
    zoom_right.set_image_size(size_right);
    zoom_right.set_zoom_factor(zoom.scale() * scale_right);
    zoom_right.set_origin(zoom.image_to_screen(&VectorD::new(size_left.width(), 0.0)));
    let pixmap_right = page_render(&page_right, &zoom_right, viewport)?;

    let surface = match (pixmap_left, pixmap_right) {
        (None, None) => return Err("empty clip".into()),
        (Some(pixmap_left), None) => SurfaceData::from_bgra8(
            pixmap_left.width() as u32,
            pixmap_left.height() as u32,
            pixmap_left.as_raw_bytes(),
        ),
        (None, Some(pixmap_right)) => SurfaceData::from_bgra8(
            pixmap_right.width() as u32,
            pixmap_right.height() as u32,
            pixmap_right.as_raw_bytes(),
        ),
        (Some(pixmap_left), Some(pixmap_right)) => {
            if pixmap_left.height() != pixmap_right.height() {
                return Err("height mismatch".into());
            }
            SurfaceData::from_dual_bgra8(
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

fn page_render(
    page: &PdfiumPage,
    zoom: &Zoom,
    viewport: &RectD,
) -> MviewResult<Option<PdfiumBitmap>> {
    let intersection = zoom.intersection(viewport);
    if intersection.is_empty() {
        Ok(None) // clip intersection is empty
    } else {
        let width = intersection.width().ceil() as i32;
        let height = intersection.height().ceil() as i32;
        let config = PdfiumRenderConfig::new()
            .with_size(width, height)
            .with_scale(zoom.scale() as f32)
            .with_pan(-intersection.x0 as f32, -intersection.y0 as f32);
        Ok(Some(page.render(&config)?))
    }
}

fn list_pages(filename: &Path) -> MviewResult<(PdfiumDocument, Vec<Row>, i32)> {
    let duration = Performance::start();
    let document = PdfiumDocument::new_from_path(filename, None)?;
    let page_count = document.page_count();
    let mut result = Vec::new();
    println!("Total pages: {page_count}");
    if page_count > 0 {
        let cat = Category::Image;
        for i in 0..page_count {
            let page = format!("Page {0:5}", i + 1);
            let row = Row {
                category: cat.id(),
                name: page,
                size: Default::default(),
                modified: Default::default(),
                index: i as u64,
                icon: cat.icon().to_string(),
                folder: Default::default(),
            };
            result.push(row);
        }
        duration.elapsed("pdfium list");
        Ok((document, result, page_count - 1))
    } else {
        Err("No pages in document".into())
    }
}
