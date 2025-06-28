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
    cmp::min,
    io::{BufRead, Seek},
    time::SystemTime,
};

use crate::{
    error::MviewResult,
    image::{animation::Animation, provider::ExifReader, Image},
};
use cairo::{Format, ImageSurface};
use gdk_pixbuf::{Pixbuf, PixbufLoader};
use glib::Bytes;
use gtk4::prelude::{PixbufAnimationExt, PixbufAnimationExtManual, PixbufLoaderExt};

pub struct GdkImageLoader {}

impl GdkImageLoader {
    pub fn image_from_reader<T: BufRead + Seek>(reader: &mut T) -> MviewResult<Image> {
        let mut buf = [0u8; 65536];
        let loader = PixbufLoader::new();
        loop {
            let num_read = reader.read(&mut buf)?;
            if num_read == 0 {
                break;
            }
            let num_read = min(num_read, buf.len());
            loader.write(&buf[0..num_read])?;
        }
        loader.close()?;
        if let Some(animation) = loader.animation() {
            if animation.is_static_image() {
                Ok(Image::new_pixbuf(animation.static_image(), reader.exif()))
            } else {
                let iter = animation.iter(Some(SystemTime::now()));
                Ok(Image::new_animation(Animation::Gdk(iter)))
            }
        } else {
            Err("No image data".into())
        }
    }

    pub fn pixbuf_from_rgb(width: u32, height: u32, rgb: &[u8]) -> Pixbuf {
        Pixbuf::from_bytes(
            &Bytes::from(rgb),
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            width as i32,
            height as i32,
            3 * width as i32,
        )
    }

    pub fn cairo_surface_from_rgb(
        width: u32,
        height: u32,
        rgb: &[u8],
    ) -> Result<ImageSurface, cairo::Error> {
        let cairo_data: Vec<u8> = rgb
            .chunks_exact(3)
            .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], 255])
            .collect();
        ImageSurface::create_for_data(
            cairo_data,
            Format::ARgb32,
            width as i32,
            height as i32,
            width as i32 * 4, // stride: bytes per row
        )
    }

    pub fn cairo_surface_from_dual_rgb(
        width_left: u32,
        width_right: u32,
        height: u32,
        rgb_left: &[u8],
        rgb_right: &[u8],
    ) -> Result<ImageSurface, cairo::Error> {
        let combined_width = width_left + width_right;
        // Create iterator that alternates between left and right pixels row by row
        let cairo_data: Vec<u8> = (0..height)
            .flat_map(|row| {
                let row_start_left = (row * width_left * 3) as usize;
                let row_end_left = row_start_left + (width_left * 3) as usize;
                let row_start_right = (row * width_right * 3) as usize;
                let row_end_right = row_start_right + (width_right * 3) as usize;
                // Combine left row + right row
                rgb_left[row_start_left..row_end_left]
                    .chunks_exact(3)
                    .chain(rgb_right[row_start_right..row_end_right].chunks_exact(3))
                    .flat_map(|chunk| [chunk[2], chunk[1], chunk[0], 255]) // RGB -> BGRA
            })
            .collect();
        ImageSurface::create_for_data(
            cairo_data,
            Format::ARgb32,
            combined_width as i32,
            height as i32,
            combined_width as i32 * 4, // stride
        )
    }
}
