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
    file_view::{BackendRef, FileRow, FileStore, ItemRef, Reference},
    image::{draw_error, SurfaceData, Zoom},
    mview6_error,
    profile::performance::Performance,
    rect::{RectD, SizeD, VectorD},
};

const MIN_DOC_HEIGHT: f32 = 32.0;

pub struct DocMuPdf {
    path: PathBuf,
    document: mupdf::Document,
    store: FileStore,
    last_page: i32,
}

impl DocMuPdf {
    pub fn try_new(filename: &Path) -> MviewResult<Self> {
        let (document, store, last_page) = Self::create_store(filename);
        Ok(DocMuPdf {
            path: filename.into(),
            document: document?,
            store,
            last_page,
        })
    }

    fn create_store(filename: &Path) -> (MviewResult<mupdf::Document>, FileStore, i32) {
        match list_pages(filename) {
            Ok((document, store, last_page)) => (Ok(document), store, last_page),
            Err(e) => {
                eprintln!("ERROR {e:?}");
                (Err(e), FileRow::empty_store(), Default::default())
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

    fn list(&self) -> FileStore {
        self.store.clone()
    }

    fn content(&self, item: &ItemRef, params: &ImageParams) -> Content {
        page_size(
            Reference {
                backend: BackendRef::Mupdf(self.path.clone()),
                item: item.clone(),
            },
            &self.document,
            item.idx() as i32,
            self.last_page,
            params.page_mode,
        )
        .map_err(|e| e.to_string())
        .unwrap_or_else(|e| draw_error(&self.path, mview6_error!(e)))
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::Mupdf(self.path.clone())
    }

    fn render(
        &self,
        item: &ItemRef,
        page_mode: &PageMode,
        zoom: &Zoom,
        viewport: &RectD,
    ) -> Option<SurfaceData> {
        render(
            &self.document,
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

    let page_right = document.load_page(index + 1)?;
    let size_right = page_size_as_rect(&page_right)?;
    let scale_right = size_left.height() / size_right.height();

    // Determine, in natural (unmirrored, un-split) spread-image coordinates, which part
    // of the dual-page spread is visible. This must be done once at the full-spread level
    // (using `zoom`'s own image_size, which covers both pages) because the mirror
    // reflection depends on the *total* spread width. Splitting first and letting each
    // page's own (smaller) Zoom clone re-derive the reflection independently -- as used to
    // be done here -- uses the wrong mirror axis per page, which only shows up once the
    // viewport doesn't cover the full spread (i.e. when zoomed in or panned), causing the
    // pages to shift/overlap instead of clipping correctly.
    let crop = zoom.intersection_image_coord(viewport);

    let crop_left = crop.intersect(&RectD::new_from_size(size_left));

    let right_rect = RectD::new(
        size_left.width(),
        0.0,
        size_left.width() + scale_right * size_right.width(),
        size_left.height(),
    );
    let crop_right = crop
        .intersect(&right_rect)
        .translate(VectorD::new(-size_left.width(), 0.0))
        .scale(1.0 / scale_right);

    let pixmap_left = page_render_crop(&page_left, &crop_left, zoom.scale())?;
    let pixmap_right = page_render_crop(&page_right, &crop_right, zoom.scale() * scale_right)?;

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
                eprintln!(
                    "Height mismatch in dual page render: left {}px, right {}px",
                    pixmap_left.height(),
                    pixmap_right.height()
                );
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

/// Renders `page` for a crop that has already been computed (in the page's own,
/// unrotated/unmirrored image coordinates) by the caller, together with the scale to
/// render it at.
///
/// Used by [`render_dual`] instead of [`page_render`], because dual-page mirroring must be
/// resolved once for the whole spread (see comment in [`render_dual`]), so by the time we
/// get here the per-page crop is already correct and needs no further mirror handling.
fn page_render_crop(page: &Page, crop: &RectD, scale: f64) -> MviewResult<Option<mupdf::Pixmap>> {
    let intersect = crop.scale(scale);

    let (x0, y0, x1, y1) = intersect.round();
    let intersect_i = IRect::new(x0, y0, x1, y1);

    if intersect_i.is_empty() {
        Ok(None) // clip intersection is empty
    } else {
        let mut pixmap = Pixmap::new_with_rect(&Colorspace::device_rgb(), intersect_i, false)?;
        pixmap.clear_with(0xff)?;

        let device = Device::from_pixmap(&pixmap)?;
        let matrix = Matrix::new_scale(scale as f32, scale as f32);
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

fn list_pages(filename: &Path) -> MviewResult<(mupdf::Document, FileStore, i32)> {
    let duration = Performance::start();
    let doc = open(filename)?;
    let page_count = doc.page_count()? as u32;
    let store = FileRow::empty_store();
    println!("Total pages: {page_count}");
    if page_count > 0 {
        let classification = FileType::Image.into();
        for i in 0..page_count {
            let page = format!("Page {0:5}", i + 1);
            store.append(&FileRow::new_index(classification, page, 0, 0, i as u64));
        }
        duration.elapsed("mupdf list");
        Ok((doc, store, page_count as i32 - 1))
    } else {
        mview6_error!("No pages in document").into()
    }
}
