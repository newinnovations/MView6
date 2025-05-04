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

use super::{Image, ImageParams};
use gtk4::ListStore;
use image::{DynamicImage, ImageBuffer, Rgb};
use mupdf::{Colorspace, Document, Matrix};
use std::{
    cell::{Cell, RefCell},
    path::Path,
};

use crate::{
    category::Category,
    error::{MviewError, MviewResult},
    file_view::{Columns, Cursor, Sort},
    image::{
        draw::draw_error,
        provider::{image_rs::RsImageLoader, ImageLoader, ImageSaver},
    },
    profile::performance::Performance,
};

use super::{
    filesystem::FileSystem,
    thumbnail::{TEntry, TReference},
    Backend, Selection,
};

#[derive(Clone, Copy, Debug, Default)]
pub enum PdfMode {
    #[default]
    Single,
    DualOdd,
    DualEven,
}

pub struct Pdf {
    filename: String,
    directory: String,
    archive: String,
    store: ListStore,
    parent: RefCell<Box<dyn Backend>>,
    sort: Cell<Sort>,
}

impl Pdf {
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
        Pdf {
            filename: filename.to_string(),
            directory: directory.to_string(),
            archive: archive.to_string(),
            store: Self::create_store(filename),
            parent: RefCell::new(<dyn Backend>::none()),
            sort: Default::default(),
        }
    }

    fn create_store(filename: &str) -> ListStore {
        println!("create_store Pdf {}", filename);
        let store = Columns::store();
        match list_pdf(filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        store
    }

    pub fn get_thumbnail(src: &TPdfReference) -> MviewResult<DynamicImage> {
        let thumb_filename = format!("{}-{}.mthumb", src.archive, src.index);
        let thumb_path = format!("{}/.mview/{}", src.directory, thumb_filename);

        if Path::new(&thumb_path).exists() {
            RsImageLoader::dynimg_from_file(&thumb_path)
        } else {
            let image = extract_pdf_thumb(&src.filename, src.index as i32)?;
            let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
            ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
            Ok(image)
        }
    }
}

impl Backend for Pdf {
    fn class_name(&self) -> &str {
        "Pdf"
    }

    fn is_container(&self) -> bool {
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

    fn image(&self, cursor: &Cursor, _: &ImageParams) -> Image {
        match extract_pdf(&self.filename, cursor.index() as i32) {
            Ok(image) => image,
            Err(error) => draw_error(error.to_string().into()),
        }
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        TEntry::new(
            cursor.category(),
            &cursor.name(),
            TReference::PdfReference(TPdfReference::new(self, cursor.index())),
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

fn extract_pdf(filename: &str, index: i32) -> Result<Image, mupdf::Error> {
    let zoom = 3.0;
    let doc = Document::open(filename)?;
    let page = doc.load_page(index)?;
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;
    Ok(ImageLoader::image_from_rgb(
        pixmap.width(),
        pixmap.height(),
        pixmap.samples(),
    ))
}

fn extract_pdf_thumb(filename: &str, index: i32) -> MviewResult<DynamicImage> {
    let zoom = 0.5;
    let doc = Document::open(filename)?;
    let page = doc.load_page(index)?;
    let matrix = Matrix::new_scale(zoom, zoom);
    let pixmap = page.to_pixmap(&matrix, &Colorspace::device_rgb(), false, false)?;

    match ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(
        pixmap.width(),
        pixmap.height(),
        pixmap.samples().to_vec(),
    ) {
        Some(rgb_image) => Ok(DynamicImage::ImageRgb8(rgb_image)),
        None => {
            // This case should ideally not be reached if the size check passes,
            // but it's good practice to handle the Option::None case.
            Err(MviewError::from(
                "Could not create ImageBuffer from pdf thumb data",
            ))
        }
    }
}

fn list_pdf(filename: &str, store: &ListStore) -> Result<(), mupdf::Error> {
    let duration = Performance::start();
    let doc = Document::open(filename)?;
    let page_count = doc.page_count()?;
    println!("Total pages: {}", page_count);
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
    duration.elapsed("list_pdf");
    Ok(())
}

// fn list_zip(filename: &str, store: &ListStore) -> ZipResult<()> {
//     let fname = std::path::Path::new(filename);
//     let file = fs::File::open(fname)?;
//     let reader = BufReader::new(file);

//     let mut archive = zip::ZipArchive::new(reader)?;

//     for i in 0..archive.len() {
//         let file = archive.by_index(i)?;

//         let outpath = match file.enclosed_name() {
//             Some(path) => path,
//             None => {
//                 println!("Entry {} has a suspicious path", file.name());
//                 continue;
//             }
//         };

//         let filename = outpath.display().to_string();
//         let cat = Category::determine(&filename, file.is_dir());
//         let file_size = file.size();
//         let index = i as u32;

//         if file_size == 0 {
//             continue;
//         }

//         if cat.id() == Category::Unsupported.id() {
//             continue;
//         }

//         let m = file.last_modified().unwrap_or_default();
//         let modified = match Local.with_ymd_and_hms(
//             m.year() as i32,
//             m.month() as u32,
//             m.day() as u32,
//             m.hour() as u32,
//             m.minute() as u32,
//             m.second() as u32,
//         ) {
//             chrono::offset::LocalResult::Single(datetime) => datetime.timestamp() as u64,
//             _ => {
//                 println!("Could not create local datetime (Ambiguous or None)");
//                 0_u64
//             }
//         };

//         store.insert_with_values(
//             None,
//             &[
//                 (Columns::Cat as u32, &cat.id()),
//                 (Columns::Icon as u32, &cat.icon()),
//                 (Columns::Name as u32, &filename),
//                 (Columns::Size as u32, &file_size),
//                 (Columns::Modified as u32, &modified),
//                 (Columns::Index as u32, &index),
//             ],
//         );
//     }
//     Ok(())
// }

#[derive(Debug, Clone)]
pub struct TPdfReference {
    filename: String,
    directory: String,
    archive: String,
    index: u32,
}

impl TPdfReference {
    pub fn new(backend: &Pdf, index: u32) -> Self {
        TPdfReference {
            filename: backend.filename.clone(),
            directory: backend.directory.clone(),
            archive: backend.archive.clone(),
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
