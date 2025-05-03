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

use super::Image;
use chrono::{Local, TimeZone};
use gtk4::ListStore;
use human_bytes::human_bytes;
use image::DynamicImage;
use std::{
    cell::{Cell, RefCell},
    fs,
    io::{BufReader, Read},
    path::Path,
};
use zip::result::ZipResult;

use crate::{
    category::Category,
    error::MviewResult,
    file_view::{Columns, Cursor, Sort},
    image::{
        draw::draw_error,
        provider::{image_rs::RsImageLoader, ImageLoader, ImageSaver},
    },
    profile::performance::Performance,
    window::MViewWidgets,
};

use super::{
    filesystem::FileSystem,
    thumbnail::{TEntry, TReference},
    Backend, Selection,
};

pub struct ZipArchive {
    filename: String,
    directory: String,
    archive: String,
    store: ListStore,
    parent: RefCell<Box<dyn Backend>>,
    sort: Cell<Sort>,
}

impl ZipArchive {
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
        ZipArchive {
            filename: filename.to_string(),
            directory: directory.to_string(),
            archive: archive.to_string(),
            store: Self::create_store(filename),
            parent: RefCell::new(<dyn Backend>::none()),
            sort: Default::default(),
        }
    }

    fn create_store(filename: &str) -> ListStore {
        println!("create_store ZipArchive {}", filename);
        let store = Columns::store();
        match list_zip(filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        store
    }

    pub fn get_thumbnail(src: &TZipReference) -> MviewResult<DynamicImage> {
        let thumb_filename = format!("{}-{}.mthumb", src.archive, src.index);
        let thumb_path = format!("{}/.mview/{}", src.directory, thumb_filename);

        if Path::new(&thumb_path).exists() {
            RsImageLoader::dynimg_from_file(&thumb_path)
        } else {
            let bytes = extract_zip(&src.filename, src.index as usize)?;
            let image = RsImageLoader::dynimg_from_memory(&bytes)?;
            let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
            ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
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

    fn image(&self, _w: &MViewWidgets, cursor: &Cursor) -> Image {
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

fn extract_zip(filename: &str, index: usize) -> ZipResult<Vec<u8>> {
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

fn list_zip(filename: &str, store: &ListStore) -> ZipResult<()> {
    let fname = std::path::Path::new(filename);
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

        let filename = outpath.display().to_string();
        let cat = Category::determine(&filename, file.is_dir());
        let file_size = file.size();
        let index = i as u32;

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

        store.insert_with_values(
            None,
            &[
                (Columns::Cat as u32, &cat.id()),
                (Columns::Icon as u32, &cat.icon()),
                (Columns::Name as u32, &filename),
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
    filename: String,
    directory: String,
    archive: String,
    index: u32,
}

impl TZipReference {
    pub fn new(backend: &ZipArchive, index: u32) -> Self {
        TZipReference {
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
