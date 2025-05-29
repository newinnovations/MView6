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

use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, ErrorKind, Read, Result, Seek, SeekFrom},
    path::Path,
};

use image::DynamicImage;

use crate::{config::contrast, error::MviewResult, image::Image};

use super::{gdk::GdkImageLoader, image_rs::RsImageLoader};

#[derive(Debug)]
pub enum ImageType {
    C,
    I,
    T,
    X,
}

#[derive(Debug)]
pub struct InternalImage {
    image_type: ImageType,
    comment: Option<String>,
    data: Vec<u8>,
}
pub struct InternalReader {}
pub struct InternalImageLoader {}

impl InternalImage {
    pub fn new<T: BufRead + Seek>(reader: &mut T, thumb: bool) -> Result<InternalImage> {
        // reader.seek(SeekFrom::Start(0))?;
        let mut buf = [0u8; 16];
        reader.read_exact(&mut buf)?;
        if &buf[0..2] != b"MP" {
            return Err(ErrorKind::Unsupported.into());
        }
        let image_type = match buf[2] {
            b'C' => ImageType::C,
            b'I' => ImageType::I,
            b'T' => ImageType::T,
            b'X' => ImageType::X,
            _ => {
                return Err(ErrorKind::InvalidData.into());
            }
        };

        // x64 and ARM64 are both little endian on Linux and Windows
        let (mode, offset, comment_length, thumb_length, image_length) = match image_type {
            ImageType::C => (
                220,
                7,
                u32::from_le_bytes(buf[3..7].try_into().unwrap()),
                0,
                0,
            ),
            ImageType::I => (220, 3, 0, 0, 0),
            ImageType::T => (
                buf[3],
                16,
                u32::from_le_bytes(buf[4..8].try_into().unwrap()),
                u32::from_le_bytes(buf[8..12].try_into().unwrap()),
                u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            ),
            ImageType::X => (
                buf[3],
                8,
                u32::from_le_bytes(buf[4..8].try_into().unwrap()),
                0,
                0,
            ),
        };

        if offset != 16 {
            reader.seek(SeekFrom::Current(offset - 16))?;
        }

        let (comment, data) = if thumb {
            if matches!(image_type, ImageType::T) {
                if thumb_length > 80_000 {
                    return Err(ErrorKind::FileTooLarge.into());
                }
                (
                    None,
                    InternalReader::read_bytes(reader, Some(thumb_length), mode)?,
                )
            } else {
                return Err(ErrorKind::Unsupported.into());
            }
        } else {
            reader.seek(SeekFrom::Current(thumb_length as i64))?;
            let comment = if comment_length > 0 {
                // reader.seek(SeekFrom::Start(offset + thumb_length as u64))?;
                let bytes = InternalReader::read_bytes(reader, Some(comment_length), mode)?;
                core::str::from_utf8(&bytes).ok().map(String::from)
            } else {
                None
            };

            if image_length > 10_000_000 {
                return Err(ErrorKind::FileTooLarge.into());
            }

            (
                comment,
                InternalReader::read_bytes(
                    reader,
                    if image_length == 0 {
                        None
                    } else {
                        Some(image_length)
                    },
                    mode,
                )?,
            )
        };

        // dbg!(&image_type, &comment);

        Ok(InternalImage {
            image_type,
            comment,
            data,
        })
    }

    #[allow(dead_code)]
    pub fn image_type(&self) -> &ImageType {
        &self.image_type
    }

    #[allow(dead_code)]
    pub fn comment(&self) -> &Option<String> {
        &self.comment
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }
}

impl InternalReader {
    pub fn read_bytes<R: Read>(reader: &mut R, length: Option<u32>, mode: u8) -> Result<Vec<u8>> {
        let mut data = match length {
            Some(length) => {
                let mut data = vec![0u8; length as usize];
                reader.read_exact(&mut data)?;
                data
            }
            None => {
                let mut data = Vec::<u8>::new();
                reader.read_to_end(&mut data)?;
                data
            }
        };
        let mode = mode + contrast();
        data.iter_mut().for_each(|data| *data ^= mode);
        Ok(data)
    }

    pub fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u64<R: Read>(reader: &mut R) -> Result<u64> {
        let mut buf = [0u8; 8];
        reader.read_exact(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }
}

impl InternalImageLoader {
    pub fn thumb_from_file(path: &Path) -> Option<DynamicImage> {
        let file = File::open(path).ok()?;
        let mut reader = BufReader::new(file);
        Self::thumb_from_reader(&mut reader).ok()
    }

    pub fn thumb_from_bytes(bytes: &[u8]) -> Option<DynamicImage> {
        let mut cursor = Cursor::new(bytes);
        Self::thumb_from_reader(&mut cursor).ok()
    }

    pub fn thumb_from_reader<T: BufRead + Seek>(reader: &mut T) -> MviewResult<DynamicImage> {
        let image = InternalImage::new(reader, true)?;
        RsImageLoader::dynimg_from_memory(image.data())
    }

    pub fn image_from_reader<T: BufRead + Seek>(reader: &mut T) -> MviewResult<Image> {
        let image = InternalImage::new(reader, false)?;
        let mut mem_reader = Cursor::new(image.data());
        let res = GdkImageLoader::image_from_reader(&mut mem_reader);
        if let Err(e) = &res {
            println!("Error {:?}", e);
        }
        res
    }
}
