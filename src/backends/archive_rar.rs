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
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use unrar::{error::UnrarError, Archive, UnrarResult};

use crate::{
    category::Category,
    content::loader::ContentLoader,
    error::MviewResult,
    file_view::{
        model::{BackendRef, ItemRef, Reference, Row},
        Cursor,
    },
    image::{
        draw::draw_error,
        provider::{image_rs::RsImageLoader, ImageSaver},
    },
    mview6_error,
    profile::performance::Performance,
};

use super::Backend;

pub struct RarArchive {
    path: PathBuf,
    store: Vec<Row>,
}

impl RarArchive {
    pub fn new(filename: &Path) -> Self {
        RarArchive {
            path: filename.into(),
            store: list_rar(filename).unwrap_or_default(),
        }
    }

    pub fn get_thumbnail(src: &Reference) -> MviewResult<DynamicImage> {
        if let (BackendRef::RarArchive(filename), ItemRef::String(selection)) = src.as_tuple() {
            if let Some(directory) = filename.parent() {
                let mut hasher = Sha256::new();
                hasher.update(filename.to_string_lossy().to_string().as_bytes());
                hasher.update(selection.as_bytes());
                let sha256sum = format!("{:x}", hasher.finalize());
                let thumb_filename = format!("{sha256sum}.mthumb");
                let thumb_path = directory.join(".mview").join(thumb_filename);

                if Path::new(&thumb_path).exists() {
                    RsImageLoader::dynimg_from_file(&thumb_path)
                } else {
                    let bytes = extract_rar(filename, selection)?;
                    let image = RsImageLoader::dynimg_from_memory(&bytes)?;
                    let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
                    ImageSaver::save_thumbnail(&thumb_path, &image);
                    Ok(image)
                }
            } else {
                mview6_error!("Failed to find directory of rar file").into() // FIXME
            }
        } else {
            mview6_error!("invalid reference").into()
        }
    }
}

impl Backend for RarArchive {
    fn class_name(&self) -> &str {
        "RarArchive"
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn list(&self) -> &Vec<Row> {
        &self.store
    }

    fn content(&self, item: &ItemRef, _: &ImageParams) -> Content {
        match extract_rar(&self.path, item.str()) {
            Ok(bytes) => ContentLoader::content_from_memory(bytes, &self.path.join(item.str())),
            Err(error) => draw_error(&self.path, error.into()),
        }
    }

    // fn content(&self, item: &ItemRef) -> Content {
    //     Content::new(
    //         Reference {
    //             backend: self.backend_ref(),
    //             item: item.clone(),
    //         },
    //         match extract_rar(&self.path, item.str()) {
    //             Ok(bytes) => ContentData::Raw(bytes),
    //             Err(error) => ContentData::Error(error.into()),
    //         },
    //     )
    // }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::RarArchive(self.path.clone())
    }

    fn item_ref(&self, cursor: &Cursor) -> ItemRef {
        ItemRef::String(cursor.name())
    }
}

fn extract_rar(rar_file: &Path, sel: &str) -> UnrarResult<Vec<u8>> {
    let duration = Performance::start();
    let mut archive = Archive::new(rar_file).open_for_processing()?;
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

fn list_rar(rar_file: &Path) -> UnrarResult<Vec<Row>> {
    let mut result = Vec::new();
    let archive = Archive::new(&rar_file).open_for_listing()?;
    for e in archive {
        let entry = e?;
        let cat = Category::determine(&entry.filename, false); //file.is_dir());
        let file_size = entry.unpacked_size;
        let modified = unix_from_msdos(entry.file_time);
        if file_size == 0 {
            continue;
        }
        if cat.id() == Category::Unsupported.id() {
            continue;
        }
        let name = entry.filename.as_os_str().to_str().unwrap_or("???");
        let row = Row {
            category: cat.id(),
            name: name.to_string(),
            size: file_size,
            modified,
            index: Default::default(),
            icon: cat.icon().to_string(),
            folder: Default::default(),
        };

        result.push(row);
    }
    Ok(result)
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
