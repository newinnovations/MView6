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
use image::DynamicImage;
use std::{
    fs,
    io::{BufReader, ErrorKind, Read, Result, Seek, SeekFrom},
    path::{Path, PathBuf},
    str::from_utf8,
};

use crate::{
    category::{ContentType, FileClassification},
    error::MviewResult,
    file_view::{
        model::{BackendRef, ItemRef, Reference, Row},
        Cursor,
    },
    image::{
        draw::draw_error,
        provider::internal::{InternalImageLoader, InternalReader},
    },
    mview6_error,
    profile::performance::Performance,
};

use super::Backend;

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
    path: PathBuf,
    store: Vec<Row>,
}

impl MarArchive {
    pub fn new(filename: &Path) -> Self {
        MarArchive {
            path: filename.into(),
            store: list_mar(filename).unwrap_or_default(),
        }
    }

    pub fn get_thumbnail(src: &Reference) -> MviewResult<DynamicImage> {
        if let (BackendRef::MarArchive(filename), ItemRef::Index(index)) = src.as_tuple() {
            dbg!(filename, index);
            let fname = Path::new(filename);
            let file = fs::File::open(fname)?;
            let mut reader = BufReader::new(file);
            reader.seek(SeekFrom::Start(*index))?;
            InternalImageLoader::thumb_from_reader(&mut reader)
        } else {
            mview6_error!("invalid reference").into()
        }
    }
}

impl Backend for MarArchive {
    fn class_name(&self) -> &str {
        "MarArchive"
    }

    fn path(&self) -> PathBuf {
        self.path.clone()
    }

    fn list(&self) -> &Vec<Row> {
        &self.store
    }

    fn content(&self, item: &ItemRef, _: &ImageParams) -> Content {
        match extract_mar(&self.path, item.idx()) {
            Ok(image) => image,
            Err(error) => draw_error(&self.path, error),
        }
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::MarArchive(self.path.clone())
    }

    fn item_ref(&self, cursor: &Cursor) -> ItemRef {
        ItemRef::Index(cursor.index())
    }
}

fn extract_mar(filename: &Path, offset: u64) -> MviewResult<Content> {
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

fn list_mar(mar_file: &Path) -> Result<Vec<Row>> {
    let mut result = Vec::new();
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

        let cat = FileClassification::determine(Path::new(&entry.filename), false);
        let file_size = entry.image_size as u64;

        if cat.content == ContentType::Unsupported {
            continue;
        }

        result.push(Row::new_index(
            cat,
            entry.filename.to_string(),
            file_size,
            entry.date,
            entry.offset,
        ));
    }
    Ok(result)
}
