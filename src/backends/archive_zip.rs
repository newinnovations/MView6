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
use chrono::{Local, TimeZone};
use gtk4::ListStore;
use human_bytes::human_bytes;
use image::DynamicImage;
use std::{
    cell::Cell,
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};
use zip::result::ZipResult;

use crate::{
    category::Category,
    error::MviewResult,
    file_view::{Columns, Cursor, Sort},
    image::{
        draw::draw_error,
        provider::{image_rs::RsImageLoader, internal::InternalImageLoader, ImageLoader},
    },
    profile::performance::Performance,
};

use super::{
    thumbnail::{TEntry, TReference},
    Backend,
};

pub struct ZipArchive {
    filename: PathBuf,
    store: ListStore,
    sort: Cell<Sort>,
}

impl ZipArchive {
    pub fn new(filename: &Path) -> Self {
        ZipArchive {
            filename: filename.into(),
            store: Self::create_store(filename),
            sort: Default::default(),
        }
    }

    fn create_store(filename: &Path) -> ListStore {
        println!("create_store ZipArchive {:?}", filename);
        let store = Columns::store();
        match list_zip(filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        store
    }

    // pub fn get_thumbnail(src: &TZipReference) -> MviewResult<DynamicImage> {
    //     let thumb_filename = format!("{}-{}.mthumb", src.archive, src.index);
    //     let thumb_path = format!("{}/.mview/{}", src.directory, thumb_filename);

    //     if Path::new(&thumb_path).exists() {
    //         RsImageLoader::dynimg_from_file(&thumb_path)
    //     } else {
    //         let bytes = extract_zip(&src.filename, src.index as usize)?;
    //         let image = RsImageLoader::dynimg_from_memory(&bytes)?;
    //         let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
    //         ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
    //         Ok(image)
    //     }
    // }

    pub fn get_thumbnail(src: &TZipReference) -> MviewResult<DynamicImage> {
        let bytes = extract_zip(&src.filename, src.index as usize)?;
        if let Some(image) = InternalImageLoader::thumb_from_bytes(&bytes) {
            Ok(image)
        } else {
            let image = RsImageLoader::dynimg_from_memory(&bytes)?;
            let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
            Ok(image)
        }
    }
}

impl Backend for ZipArchive {
    fn class_name(&self) -> &str {
        "ZipArchive"
    }

    fn is_container(&self) -> bool {
        true
    }

    fn path(&self) -> PathBuf {
        self.filename.clone()
    }

    fn store(&self) -> ListStore {
        self.store.clone()
    }

    fn image(&self, cursor: &Cursor, _: &ImageParams) -> Image {
        match extract_zip(&self.filename, cursor.index() as usize) {
            Ok(bytes) => {
                ImageLoader::image_from_memory(bytes, cursor.name().to_lowercase().contains(".svg"))
            }
            Err(error) => draw_error(error.into()),
        }
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        TEntry::new(
            cursor.category(),
            &cursor.name(),
            TReference::ZipReference(TZipReference::new(self, cursor.index())),
        )
    }

    fn set_sort(&self, sort: &Sort) {
        self.sort.set(*sort)
    }

    fn sort(&self) -> Sort {
        self.sort.get()
    }
}

fn extract_zip(filename: &Path, index: usize) -> ZipResult<Vec<u8>> {
    let duration = Performance::start();
    let fname = std::path::Path::new(filename);
    let file = fs::File::open(fname)?;
    let reader = BufReader::new(file);
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut file = archive.by_index(index)?;
    let mut buf = Vec::<u8>::new();
    let size = file.read_to_end(&mut buf)?;
    duration.elapsed_suffix("extract (zip)", &format!("({})", &human_bytes(size as f64)));
    Ok(buf)
}

fn list_zip(zip_file: &Path, store: &ListStore) -> ZipResult<()> {
    let fname = std::path::Path::new(zip_file);
    let file = fs::File::open(fname)?;
    let reader = BufReader::new(file);

    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;

        let outpath = match file.enclosed_name() {
            Some(path) => path,
            None => {
                println!("Entry {} has a suspicious path", file.name());
                continue;
            }
        };

        let cat = Category::determine(&outpath, file.is_dir());
        let file_size = file.size();
        let index = i as u64;

        if file_size == 0 {
            continue;
        }

        if cat.id() == Category::Unsupported.id() {
            continue;
        }

        let m = file.last_modified().unwrap_or_default();
        let modified = match Local.with_ymd_and_hms(
            m.year() as i32,
            m.month() as u32,
            m.day() as u32,
            m.hour() as u32,
            m.minute() as u32,
            m.second() as u32,
        ) {
            chrono::offset::LocalResult::Single(datetime) => datetime.timestamp() as u64,
            _ => {
                println!("Could not create local datetime (Ambiguous or None)");
                0_u64
            }
        };

        let name = outpath
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        store.insert_with_values(
            None,
            &[
                (Columns::Cat as u32, &cat.id()),
                (Columns::Icon as u32, &cat.icon()),
                (Columns::Name as u32, &name),
                (Columns::Size as u32, &file_size),
                (Columns::Modified as u32, &modified),
                (Columns::Index as u32, &index),
            ],
        );
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct TZipReference {
    filename: PathBuf,
    index: u64,
}

impl TZipReference {
    pub fn new(backend: &ZipArchive, index: u64) -> Self {
        TZipReference {
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
