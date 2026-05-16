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

use image::{codecs::jpeg::JpegEncoder, ColorType, DynamicImage};
use std::io::{BufRead, ErrorKind, Result, Write};

use crate::error::MviewResult;

const TARGET_WIDTH: u32 = 1024;
const TARGET_HEIGHT: u32 = 1024;
const QUALITY: u8 = 70;

pub struct PreviewImage {
    width: u32,
    height: u32,
    caption: String,
    image_jpeg: Vec<u8>,
}

impl PreviewImage {
    pub fn new(img: DynamicImage, caption: String) -> MviewResult<Self> {
        let (orig_width, orig_height) = (img.width(), img.height());

        // Calculate scaling factor to fit within target dimensions while preserving aspect ratio
        let width_scale = TARGET_WIDTH as f64 / orig_width as f64;
        let height_scale = TARGET_HEIGHT as f64 / orig_height as f64;
        let scale = width_scale.min(height_scale);
        let width = (orig_width as f64 * scale) as u32;
        let height = (orig_height as f64 * scale) as u32;

        // Resize the image
        let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

        let mut image_jpeg = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut image_jpeg, QUALITY);
        encoder.encode(
            &resized.to_rgb8().into_raw(),
            width,
            height,
            ColorType::Rgb8.into(),
        )?;

        Ok(Self {
            width,
            height,
            caption,
            image_jpeg,
        })
    }

    // 4 total length
    // 4 width
    // 4 height
    // 4 length caption
    // 4 length jpeg encoding
    // n bytes of caption
    // n bytes of jpeg encoding

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let caption_bytes = self.caption.clone().into_bytes();

        let total_length = 5 * 4 + caption_bytes.len() + self.image_jpeg.len();
        writer.write_all(&(total_length as u32).to_le_bytes())?; // 4

        writer.write_all(&self.width.to_le_bytes())?; // 4
        writer.write_all(&self.height.to_le_bytes())?; // 4

        writer.write_all(&(caption_bytes.len() as u32).to_le_bytes())?; // 4
        writer.write_all(&(self.image_jpeg.len() as u32).to_le_bytes())?; // 4

        writer.write_all(&caption_bytes)?;
        writer.write_all(&self.image_jpeg)?;

        Ok(())
    }

    pub fn read<T: BufRead>(reader: &mut T) -> Result<Self> {
        let mut buf = [0u8; 20];
        reader.read_exact(&mut buf)?;

        let total_length = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let width = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let height = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let caption_length = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let jpeg_length = u32::from_le_bytes(buf[16..20].try_into().unwrap());

        // Sanity check on values
        if total_length > 5_000_000
            || jpeg_length > 4_000_000
            || width > 4096
            || height > 4096
            || caption_length > 16_384
        {
            return Err(ErrorKind::FileTooLarge.into());
        }

        if total_length != 5 * 4 + caption_length + jpeg_length {
            return Err(ErrorKind::InvalidData.into());
        }

        let mut caption = vec![0u8; caption_length as usize];
        reader.read_exact(&mut caption)?;

        let mut image_jpeg = vec![0u8; jpeg_length as usize];
        reader.read_exact(&mut image_jpeg)?;

        Ok(Self {
            width,
            height,
            caption: String::from_utf8(caption).unwrap_or_default(),
            image_jpeg,
        })
    }

    pub fn jpeg_data(&self) -> &[u8] {
        &self.image_jpeg
    }
}
