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
use gtk4::ListStore;
use image::DynamicImage;
use std::{
    fs,
    io::{BufReader, ErrorKind, Read, Result, Seek, SeekFrom},
    path::{Path, PathBuf},
    str::from_utf8,
};

use crate::{
    category::Category,
    error::MviewResult,
    file_view::{Column, Cursor},
    image::{
        draw::draw_error,
        provider::internal::{InternalImageLoader, InternalReader},
    },
    profile::performance::Performance,
};

use super::{
    thumbnail::{TEntry, TReference},
    Backend,
};

pub struct MarEntry {
    pub offset: u64,
    pub filename: String,
    pub image_size: u32,
    pub date: u64,
}

impl MarEntry {
    pub fn read<R: Read>(reader: &mut R, mode: u8) -> Result<Self> {
        let _length = InternalReader::read_u32(reader)?;
        let offset = InternalReader::read_u64(reader)?;
        let image_size = InternalReader::read_u32(reader)?;
        let date = InternalReader::read_u64(reader)?;
        let filename_length = InternalReader::read_u32(reader)?;
        let filename_bytes = InternalReader::read_bytes(reader, Some(filename_length), mode)?;
        let filename = from_utf8(&filename_bytes).unwrap_or_default().to_string();
        Ok(MarEntry {
            offset,
            filename,
            image_size,
            date,
        })
    }
}

pub struct MarArchive {
    filename: PathBuf,
    store: ListStore,
}

impl MarArchive {
    pub fn new(filename: &Path) -> Self {
        MarArchive {
            filename: filename.into(),
            store: Self::create_store(filename),
        }
    }

    fn create_store(filename: &Path) -> ListStore {
        println!("create_store MarArchive {:?}", filename);
        let store = Column::empty_store();
        match list_mar(filename, &store) {
            Ok(()) => println!("OK"),
            Err(e) => println!("ERROR {:?}", e),
        };
        store
    }

    pub fn get_thumbnail(src: &TMarReference) -> MviewResult<DynamicImage> {
        let fname = Path::new(&src.filename);
        let file = fs::File::open(fname)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(src.index))?;
        InternalImageLoader::thumb_from_reader(&mut reader)
    }
}

impl Backend for MarArchive {
    fn class_name(&self) -> &str {
        "MarArchive"
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
        match extract_mar(&self.filename, cursor.index()) {
            Ok(image) => image,
            Err(error) => draw_error(error),
        }
    }

    fn entry(&self, cursor: &Cursor) -> TEntry {
        TEntry::new(
            cursor.category(),
            &cursor.name(),
            TReference::MarReference(TMarReference::new(self, cursor.index())),
        )
    }
}

fn extract_mar(filename: &Path, offset: u64) -> MviewResult<Image> {
    let duration = Performance::start();
    let fname = std::path::Path::new(filename);
    let file = fs::File::open(fname)?;
    let mut reader = BufReader::new(file);
    // println!("Offset {}", offset);
    reader.seek(SeekFrom::Start(offset))?;
    let image = InternalImageLoader::image_from_reader(&mut reader);
    duration.elapsed("extract/dec (mar)");
    image
}

fn list_mar(mar_file: &Path, store: &ListStore) -> Result<()> {
    let fname = std::path::Path::new(mar_file);
    let file = fs::File::open(fname)?;
    let mut reader = BufReader::new(file);

    let mut buf = [0u8; 12];
    reader.read_exact(&mut buf)?;
    if &buf[0..4] != b"MAR2" {
        return Err(ErrorKind::Unsupported.into());
    }
    let start_of_directory = u64::from_le_bytes(buf[4..12].try_into().unwrap());
    reader.seek(SeekFrom::Start(start_of_directory))?;
    if InternalReader::read_bytes(&mut reader, Some(4), buf[3])? != b"DIR2" {
        return Err(ErrorKind::Unsupported.into());
    }
    let num_entries = InternalReader::read_u32(&mut reader)?;

    for _ in 0..num_entries {
        let entry = MarEntry::read(&mut reader, buf[3])?;

        let cat = Category::determine(Path::new(&entry.filename), false);
        let file_size = entry.image_size as u64;

        if cat.id() == Category::Unsupported.id() {
            continue;
        }

        store.insert_with_values(
            None,
            &[
                (Column::Cat as u32, &cat.id()),
                (Column::Icon as u32, &cat.icon()),
                (Column::Name as u32, &entry.filename),
                (Column::Size as u32, &file_size),
                (Column::Modified as u32, &entry.date),
                (Column::Index as u32, &entry.offset),
            ],
        );
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct TMarReference {
    filename: PathBuf,
    index: u64,
}

impl TMarReference {
    pub fn new(backend: &MarArchive, index: u64) -> Self {
        TMarReference {
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
