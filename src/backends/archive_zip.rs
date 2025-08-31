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

use super::{Content, ImageParams};
use chrono::{Local, TimeZone};
use human_bytes::human_bytes;
use image::DynamicImage;
use std::{
    fs,
    io::{BufReader, Read},
    path::{Path, PathBuf},
};
use zip::result::ZipResult;

use crate::{
    category::Category,
    error::MviewResult,
    file_view::{
        model::{BackendRef, ItemRef, Reference, Row},
        Cursor,
    },
    image::{
        draw::draw_error,
        provider::{image_rs::RsImageLoader, internal::InternalImageLoader, ImageLoader},
    },
    profile::performance::Performance,
    util::path_to_filename,
};

use super::Backend;

pub struct ZipArchive {
    path: PathBuf,
    store: Vec<Row>,
}

impl ZipArchive {
    pub fn new(filename: &Path) -> Self {
        ZipArchive {
            path: filename.into(),
            store: list_zip(filename).unwrap_or_default(),
        }
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

    pub fn get_thumbnail(src: &Reference) -> MviewResult<DynamicImage> {
        if let (BackendRef::ZipArchive(filename), ItemRef::Index(index)) = src.as_tuple() {
            let bytes = extract_zip(filename, *index as usize)?;
            if let Some(image) = InternalImageLoader::thumb_from_bytes(&bytes) {
                Ok(image)
            } else {
                let image = RsImageLoader::dynimg_from_memory(&bytes)?;
                let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
                Ok(image)
            }
        } else {
            Err("invalid reference".into())
        }
    }
}

impl Backend for ZipArchive {
    fn class_name(&self) -> &str {
        "ZipArchive"
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn store(&self) -> &Vec<Row> {
        &self.store
    }

    fn image(&self, item: &ItemRef, _: &ImageParams) -> Content {
        match extract_zip(&self.path, item.idx() as usize) {
            Ok(bytes) => ImageLoader::image_from_memory(bytes),
            Err(error) => draw_error(error.into()),
        }
    }

    // fn content(&self, item: &ItemRef) -> Content {
    //     Content::new(
    //         Reference {
    //             backend: self.backend_ref(),
    //             item: item.clone(),
    //         },
    //         match extract_zip(&self.path, item.idx() as usize) {
    //             Ok(bytes) => ContentData::Raw(bytes),
    //             Err(error) => ContentData::Error(error.into()),
    //         },
    //     )
    // }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::ZipArchive(self.path.clone())
    }

    fn item_ref(&self, cursor: &Cursor) -> ItemRef {
        ItemRef::Index(cursor.index())
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

fn list_zip(zip_file: &Path) -> ZipResult<Vec<Row>> {
    let mut result = Vec::new();
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

        let row = Row {
            category: cat.id(),
            name: path_to_filename(&outpath),
            size: file_size,
            modified,
            index,
            icon: cat.icon().to_string(),
            folder: Default::default(),
        };

        result.push(row);
    }
    Ok(result)
}
