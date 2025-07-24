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

use std::{
    cmp::min,
    io::{BufRead, Seek},
    slice,
    time::SystemTime,
};

use crate::{
    error::MviewResult,
    image::{
        animation::Animation,
        provider::{surface::convert_rgba_pixel, ExifReader},
        Image,
    },
    profile::performance::Performance,
};
use cairo::{Format, ImageSurface};
use gdk_pixbuf::{
    ffi::{gdk_pixbuf_get_byte_length, gdk_pixbuf_read_pixels},
    Pixbuf, PixbufLoader,
};
use glib::translate::ToGlibPtr;
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

    pub fn surface_from_rgb(
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
            Format::Rgb24,
            width as i32,
            height as i32,
            width as i32 * 4, // stride: bytes per row
        )
    }

    pub fn surface_from_dual_rgb(
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
            Format::Rgb24,
            combined_width as i32,
            height as i32,
            combined_width as i32 * 4, // stride
        )
    }

    // https://users.rust-lang.org/t/converting-a-bgra-u8-to-rgb-u8-n-for-images/67938
    pub fn surface_from_pixbuf(p: &Pixbuf) -> Result<ImageSurface, cairo::Error> {
        let duration = Performance::start();

        let width = p.width() as usize;
        let height = p.height() as usize;
        let pixbuf_stride = p.rowstride() as usize;

        // Both ARgb32 and Rgb24 take 4 bytes per pixel for performace reasons
        let surface_stride = 4 * width;
        let mut surface_data = vec![0_u8; height * surface_stride];

        let format = unsafe {
            // gain access without the copy of pixbuf memory
            let pixbuf_data_raw = gdk_pixbuf_read_pixels(p.to_glib_none().0);
            let pixbuf_data_len = gdk_pixbuf_get_byte_length(p.to_glib_none().0);
            let pixbuf_data = slice::from_raw_parts(pixbuf_data_raw, pixbuf_data_len);

            if p.has_alpha() {
                for (src_row, dst_row) in pixbuf_data
                    .chunks_exact(pixbuf_stride)
                    .zip(surface_data.chunks_exact_mut(surface_stride))
                {
                    for (src, dst) in src_row.chunks_exact(4).zip(dst_row.chunks_exact_mut(4)) {
                        convert_rgba_pixel(src, dst);
                    }
                }
                Format::ARgb32
            } else {
                for (src_row, dst_row) in pixbuf_data
                    .chunks_exact(pixbuf_stride)
                    .zip(surface_data.chunks_exact_mut(surface_stride))
                {
                    for (src, dst) in src_row.chunks_exact(3).zip(dst_row.chunks_exact_mut(4)) {
                        dst[0] = src[2];
                        dst[1] = src[1];
                        dst[2] = src[0];
                    }
                }
                Format::Rgb24
            }
        };

        let surface = ImageSurface::create_for_data(
            surface_data,
            format,
            width as i32,
            height as i32,
            surface_stride as i32,
        );

        duration.elapsed("surface");

        surface
    }

    pub fn surface_from_pixbuf_option(p: Option<&Pixbuf>) -> Option<ImageSurface> {
        p.map(Self::surface_from_pixbuf).and_then(Result::ok)
    }
}

// pub fn debug_stride(format: Format) {
//     for w in 100..108 {
//         if let Ok(stride) = format.stride_for_width(w) {
//             let per = stride as f64 / w as f64;
//             println!("{format:?} {w} {stride} {per}")
//         }
//     }
// }

// pub fn debug_strides() {
//     debug_stride(Format::ARgb32);
//     debug_stride(Format::Rgb24);
//     debug_stride(Format::Rgb30);
//     debug_stride(Format::Rgb16_565);
//     debug_stride(Format::A1);
//     debug_stride(Format::A8);
// }

// ARgb32 100 400 4
// ARgb32 101 404 4
// ARgb32 102 408 4
// ARgb32 103 412 4
// ARgb32 104 416 4
// ARgb32 105 420 4
// ARgb32 106 424 4
// ARgb32 107 428 4
// Rgb24 100 400 4
// Rgb24 101 404 4
// Rgb24 102 408 4
// Rgb24 103 412 4
// Rgb24 104 416 4
// Rgb24 105 420 4
// Rgb24 106 424 4
// Rgb24 107 428 4
// Rgb30 100 400 4
// Rgb30 101 404 4
// Rgb30 102 408 4
// Rgb30 103 412 4
// Rgb30 104 416 4
// Rgb30 105 420 4
// Rgb30 106 424 4
// Rgb30 107 428 4
// Rgb16_565 100 200 2
// Rgb16_565 101 204 2.01980198019802
// Rgb16_565 102 204 2
// Rgb16_565 103 208 2.0194174757281553
// Rgb16_565 104 208 2
// Rgb16_565 105 212 2.019047619047619
// Rgb16_565 106 212 2
// Rgb16_565 107 216 2.0186915887850465
// A1 100 16 0.16
// A1 101 16 0.15841584158415842
// A1 102 16 0.1568627450980392
// A1 103 16 0.1553398058252427
// A1 104 16 0.15384615384615385
// A1 105 16 0.1523809523809524
// A1 106 16 0.1509433962264151
// A1 107 16 0.14953271028037382
// A8 100 100 1
// A8 101 104 1.0297029702970297
// A8 102 104 1.0196078431372548
// A8 103 104 1.0097087378640777
// A8 104 104 1
// A8 105 108 1.0285714285714285
// A8 106 108 1.0188679245283019
// A8 107 108 1.0093457943925233
