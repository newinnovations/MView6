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
use sha2::{Digest, Sha256};
use std::{
    cell::{Cell, RefCell},
    path::Path,
};
use unrar::{error::UnrarError, Archive, UnrarResult};

use crate::{
    category::Category,
    error::MviewResult,
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
    Backend, Target,
};

pub struct RarArchive {
    filename: String,
    directory: String,
    archive: String,
    store: ListStore,
    parent: RefCell<Box<dyn Backend>>,
    sort: Cell<Sort>,
}

impl RarArchive {
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
        RarArchive {
            filename: filename.to_string(),
            directory: directory.to_string(),
            archive: archive.to_string(),
            store: Self::create_store(filename),
            parent: RefCell::new(<dyn Backend>::none()),
            sort: Default::default(),
        }
    }

    fn create_store(filename: &str) -> ListStore {
        let store = Columns::store();
        match list_rar(filename, &store) {
            Ok(()) => (),
            Err(e) => println!("ERROR {:?}", e),
        };
        store
    }

    pub fn get_thumbnail(src: &TRarReference) -> MviewResult<DynamicImage> {
        let mut hasher = Sha256::new();
        hasher.update(src.archive.as_bytes());
        hasher.update(src.selection.as_bytes());
        let sha256sum = format!("{:x}", hasher.finalize());
        let thumb_filename = format!("{sha256sum}.mthumb");
        let thumb_path = format!("{}/.mview/{}", src.directory, thumb_filename);

        if Path::new(&thumb_path).exists() {
            RsImageLoader::dynimg_from_file(&thumb_path)
        } else {
            let bytes = extract_rar(&src.filename, &src.selection)?;
            let image = RsImageLoader::dynimg_from_memory(&bytes)?;
            let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
            ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
            Ok(image)
        }
    }
}

impl Backend for RarArchive {
    fn class_name(&self) -> &str {
        "RarArchive"
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

    fn leave(&self) -> (Box<dyn Backend>, Target) {
        if self.parent.borrow().is_none() {
            (
                Box::new(FileSystem::new(&self.directory)),
                Target::Name(self.archive.clone()),
            )
        } else {
            (
                self.parent.replace(<dyn Backend>::none()),
                Target::Name(self.archive.clone()),
            )
        }
    }

    fn image(&self, cursor: &Cursor, _: &ImageParams) -> Image {
        let sel = cursor.name();
        match extract_rar(&self.filename, &sel) {
            Ok(bytes) => ImageLoader::image_from_memory(bytes, sel.to_lowercase().contains(".svg")),
            Err(error) => draw_error(error.into()),
        }
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        TEntry::new(
            cursor.category(),
            &cursor.name(),
            TReference::RarReference(TRarReference::new(self, &cursor.name())),
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

fn extract_rar(filename: &str, sel: &str) -> UnrarResult<Vec<u8>> {
    let duration = Performance::start();
    let mut archive = Archive::new(filename).open_for_processing()?;
    while let Some(header) = archive.read_header()? {
        let e_filename = header.entry().filename.as_os_str().to_str().unwrap_or("-");
        archive = if header.entry().is_file() {
            if e_filename == sel {
                let (bytes, _) = header.read()?;
                duration.elapsed_suffix(
                    "extract (rar)",
                    &format!("({})", &human_bytes(bytes.len() as f64)),
                );
                return Ok(bytes);
            } else {
                header.skip()?
            }
        } else {
            header.skip()?
        };
    }
    Err(UnrarError {
        code: unrar::error::Code::EndArchive,
        when: unrar::error::When::Read,
    })
}

fn list_rar(filename: &str, store: &ListStore) -> UnrarResult<()> {
    let archive = Archive::new(&filename).open_for_listing()?;
    for e in archive {
        let entry = e?;
        let filename = entry.filename.as_os_str().to_str().unwrap_or("???");
        let cat = Category::determine(filename, false); //file.is_dir());
        let file_size = entry.unpacked_size;
        let modified = unix_from_msdos(entry.file_time);
        if file_size == 0 {
            continue;
        }
        if cat.id() == Category::Unsupported.id() {
            continue;
        }
        store.insert_with_values(
            None,
            &[
                (Columns::Cat as u32, &cat.id()),
                (Columns::Icon as u32, &cat.icon()),
                (Columns::Name as u32, &filename),
                (Columns::Size as u32, &file_size),
                (Columns::Modified as u32, &modified),
            ],
        );
    }
    Ok(())
}

pub fn unix_from_msdos(dostime: u32) -> u64 {
    let second = (dostime & 0b0000000000011111) << 1;
    let minute = (dostime & 0b0000011111100000) >> 5;
    let hour = (dostime & 0b1111100000000000) >> 11;

    let datepart = dostime >> 16;
    let day = datepart & 0b0000000000011111;
    let month = (datepart & 0b0000000111100000) >> 5;
    let year = 1980 + ((datepart & 0b1111111000000000) >> 9);

    match Local.with_ymd_and_hms(year as i32, month, day, hour, minute, second) {
        chrono::offset::LocalResult::Single(datetime) => datetime.timestamp() as u64,
        _ => {
            println!("Could not create local datetime (Ambiguous or None)");
            0_u64
        }
    }
}

#[derive(Debug, Clone)]
pub struct TRarReference {
    filename: String,
    directory: String,
    archive: String,
    selection: String,
}

impl TRarReference {
    pub fn new(backend: &RarArchive, selection: &str) -> Self {
        TRarReference {
            filename: backend.filename.clone(),
            directory: backend.directory.clone(),
            archive: backend.archive.clone(),
            selection: selection.to_string(),
        }
    }

    pub fn selection(&self) -> String {
        self.selection.clone()
    }
}
