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

use image::{DynamicImage, ImageBuffer, Rgb};
use mupdf::{Colorspace, Device, IRect, Matrix, Page, Pixmap, Rect};
use std::path::{Path, PathBuf};

use crate::{
    backends::{
        document::{pages, PageMode, Pages},
        Backend, ImageParams,
    },
    classification::FileType,
    content::Content,
    error::MviewResult,
    file_view::{
        model::{BackendRef, ItemRef, Reference, Row},
        Cursor,
    },
    image::{draw::draw_error, provider::surface::SurfaceData, view::Zoom},
    mview6_error,
    profile::performance::Performance,
    rect::{RectD, SizeD, VectorD},
};

const MIN_DOC_HEIGHT: f32 = 32.0;

pub struct DocMuPdf {
    path: PathBuf,
    document: MviewResult<mupdf::Document>,
    store: Vec<Row>,
    last_page: i32,
}

impl DocMuPdf {
    pub fn new(filename: &Path) -> Self {
        let (document, store, last_page) = Self::create_store(filename);
        DocMuPdf {
            path: filename.into(),
            document,
            store,
            last_page,
        }
    }

    fn create_store(filename: &Path) -> (MviewResult<mupdf::Document>, Vec<Row>, i32) {
        match list_pages(filename) {
            Ok((document, store, last_page)) => (Ok(document), store, last_page),
            Err(e) => {
                eprintln!("ERROR {e:?}");
                (Err(e), Default::default(), Default::default())
            }
        }
    }

    pub fn get_thumbnail(src: &Reference) -> MviewResult<DynamicImage> {
        if let (BackendRef::Mupdf(filename), ItemRef::Index(index)) = src.as_tuple() {
            let image = extract_thumb(filename, *index as i32)?;
            let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
            Ok(image)
        } else {
            mview6_error!("invalid reference").into()
        }
    }
}

impl Backend for DocMuPdf {
    fn class_name(&self) -> &str {
        "MuPDF"
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
                    backend: BackendRef::Mupdf(self.path.clone()),
                    item: item.clone(),
                },
                document,
                item.idx() as i32,
                self.last_page,
                params.page_mode,
            )
            .map_err(|e| e.to_string())
        })()
        .unwrap_or_else(|e| draw_error(&self.path, mview6_error!(e)))
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::Mupdf(self.path.clone())
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
    document: &mupdf::Document,
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
    document: &mupdf::Document,
    index: i32,
) -> MviewResult<Content> {
    let duration = Performance::start();
    let size = page_size_as_rect(&document.load_page(index)?)?;
    let image = Content::new_doc(reference, *mode, size);
    duration.elapsed("mupdf single");
    Ok(image)
}

fn page_size_dual(
    reference: Reference,
    mode: &PageMode,
    document: &mupdf::Document,
    index: i32,
) -> MviewResult<Content> {
    // The right page is scaled so its height is the same as the left page
    let duration = Performance::start();
    let size_left = page_size_as_rect(&document.load_page(index)?)?;
    let size_right = page_size_as_rect(&document.load_page(index + 1)?)?;
    let scale_right = size_left.height() / size_right.height();
    let size = SizeD::new(
        size_left.width() + scale_right * size_right.width(),
        size_left.height(),
    );
    let image = Content::new_doc(reference, *mode, size);
    duration.elapsed("mupdf dual");
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
        None => mview6_error!("Could not create ImageBuffer from pdf thumb data").into(),
    }
}

fn page_size_as_rect(page: &Page) -> MviewResult<SizeD> {
    let bounds = page.bounds()?;
    Ok(SizeD::new(bounds.width() as f64, bounds.height() as f64))
}

fn render(
    document: &mupdf::Document,
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
    document: &mupdf::Document,
    index: i32,
    zoom: &Zoom,
    viewport: &RectD,
) -> MviewResult<SurfaceData> {
    let duration = Performance::start();
    let page = document.load_page(index)?;
    let surface = if let Some(pixmap) = page_render(&page, zoom, viewport)? {
        Ok(SurfaceData::from_rgb(
            pixmap.width(),
            pixmap.height(),
            pixmap.samples(),
        ))
    } else {
        mview6_error!("empty clip").into()
    };
    duration.elapsed("mupdf clip:1");
    surface
}

fn render_dual(
    document: &mupdf::Document,
    index: i32,
    zoom: &Zoom,
    viewport: &RectD,
) -> MviewResult<SurfaceData> {
    let duration = Performance::start();

    let page_left = document.load_page(index)?;
    let size_left = page_size_as_rect(&page_left)?;
    let mut zoom_left = zoom.clone();
    zoom_left.set_image_size(size_left);
    let pixmap_left = page_render(&page_left, &zoom_left, viewport)?;

    let page_right = document.load_page(index + 1)?;
    let size_right = page_size_as_rect(&page_right)?;
    let scale_right = size_left.height() / size_right.height();
    let mut zoom_right = zoom.clone();
    zoom_right.set_image_size(size_right);
    zoom_right.set_zoom_factor(zoom.scale() * scale_right);
    zoom_right.set_origin(zoom.image_to_screen(&VectorD::new(size_left.width(), 0.0)));
    let pixmap_right = page_render(&page_right, &zoom_right, viewport)?;

    let surface = match (pixmap_left, pixmap_right) {
        (None, None) => return mview6_error!("empty clip").into(),
        (Some(pixmap_left), None) => SurfaceData::from_rgb(
            pixmap_left.width(),
            pixmap_left.height(),
            pixmap_left.samples(),
        ),
        (None, Some(pixmap_right)) => SurfaceData::from_rgb(
            pixmap_right.width(),
            pixmap_right.height(),
            pixmap_right.samples(),
        ),
        (Some(pixmap_left), Some(pixmap_right)) => {
            if pixmap_left.height() != pixmap_right.height() {
                return mview6_error!("height mismatch").into();
            }
            SurfaceData::from_dual_rgb(
                pixmap_left.width(),
                pixmap_right.width(),
                pixmap_left.height(),
                pixmap_left.samples(),
                pixmap_right.samples(),
            )
        }
    };

    duration.elapsed("mupdf clip:2");
    Ok(surface)
}

fn open_page(doc: &mupdf::Document, page_no: i32) -> MviewResult<(Page, Rect)> {
    let page = doc.load_page(page_no)?;
    let bounds = page.bounds()?;
    if bounds.height() < MIN_DOC_HEIGHT {
        return mview6_error!("page height too small").into();
    }
    Ok((page, bounds))
}

fn page_render(page: &Page, zoom: &Zoom, viewport: &RectD) -> MviewResult<Option<mupdf::Pixmap>> {
    let intersect = zoom.intersection(viewport);

    let (x0, y0, x1, y1) = intersect.round();
    let intersect_i = IRect::new(x0, y0, x1, y1);

    if intersect_i.is_empty() {
        Ok(None) // clip intersection is empty
    } else {
        let mut pixmap = Pixmap::new_with_rect(&Colorspace::device_rgb(), intersect_i, false)?;
        pixmap.clear_with(0xff)?;

        let device = Device::from_pixmap(&pixmap)?;
        let matrix = Matrix::new_scale(zoom.scale() as f32, zoom.scale() as f32);
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

fn list_pages(filename: &Path) -> MviewResult<(mupdf::Document, Vec<Row>, i32)> {
    let duration = Performance::start();
    let doc = open(filename)?;
    let page_count = doc.page_count()? as u32;
    let mut result = Vec::new();
    println!("Total pages: {page_count}");
    if page_count > 0 {
        let cat = FileType::Image.into();
        for i in 0..page_count {
            let page = format!("Page {0:5}", i + 1);
            result.push(Row::new_index(cat, page, 0, 0, i as u64));
        }
        duration.elapsed("mupdf list");
        Ok((doc, result, page_count as i32 - 1))
    } else {
        mview6_error!("No pages in document").into()
    }
}
