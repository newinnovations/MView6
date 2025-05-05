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
    io::{BufRead, BufReader, Cursor, Seek},
};

use exif::Exif;
use gdk_pixbuf::Pixbuf;
use glib::Bytes;
use image::{DynamicImage, GenericImageView, ImageReader};

use crate::{error::MviewResult, image::Image};

use super::{webp::WebP, ExifReader};

pub struct RsImageLoader {}

impl RsImageLoader {
    pub fn dynimg_from_memory(buffer: &Vec<u8>) -> MviewResult<DynamicImage> {
        Self::dynimg(ImageReader::new(Cursor::new(buffer)))
    }

    pub fn dynimg_from_file(filename: &str) -> MviewResult<DynamicImage> {
        Self::dynimg(ImageReader::open(filename)?)
    }
}

impl RsImageLoader {
    pub fn image_from_file(mut reader: BufReader<File>) -> MviewResult<Image> {
        let exif = reader.exif();
        let image_reader = ImageReader::new(reader);
        let image_reader = image_reader.with_guessed_format()?;
        if let Some(format) = image_reader.format() {
            match format {
                image::ImageFormat::WebP => WebP::image_from_file(image_reader.into_inner(), exif),
                _ => Self::image(image_reader, exif),
            }
        } else {
            Err("Unrecognized image format".into())
        }
    }

    pub fn image_from_memory(mut reader: Cursor<Vec<u8>>) -> MviewResult<Image> {
        let exif = reader.exif();
        let image_reader = ImageReader::new(reader);
        let image_reader = image_reader.with_guessed_format()?;
        if let Some(format) = image_reader.format() {
            match format {
                image::ImageFormat::WebP => {
                    WebP::image_from_memory(image_reader.into_inner(), exif)
                }
                _ => Self::image(image_reader, exif),
            }
        } else {
            Err("Unrecognized image format".into())
        }
    }
}

impl RsImageLoader {
    pub fn image<T: BufRead + Seek>(
        reader: ImageReader<T>,
        exif: Option<Exif>,
    ) -> MviewResult<Image> {
        Ok(Image::new_pixbuf(Some(Self::pixbuf(reader)?), exif))
    }

    pub fn pixbuf<T: BufRead + Seek>(reader: ImageReader<T>) -> MviewResult<Pixbuf> {
        let reader = reader.with_guessed_format()?;
        let dynamic_image = reader.decode()?;
        Self::dynimg_to_pixbuf(dynamic_image)
    }

    pub fn dynimg<T: BufRead + Seek>(reader: ImageReader<T>) -> MviewResult<DynamicImage> {
        let reader = reader.with_guessed_format()?;
        Ok(reader.decode()?)
    }
}

impl RsImageLoader {
    pub fn dynimg_to_pixbuf(image: DynamicImage) -> MviewResult<Pixbuf> {
        let (width, height) = image.dimensions();
        let colorspace;
        let has_alpha;
        let bits_per_sample;
        let rowstride;

        let image = match image.color() {
            image::ColorType::L8 => DynamicImage::from(image.to_rgb8()),
            image::ColorType::La8 => DynamicImage::from(image.to_rgba8()),
            image::ColorType::L16 => DynamicImage::from(image.to_rgb8()),
            image::ColorType::La16 => DynamicImage::from(image.to_rgba8()),
            image::ColorType::Rgb16 => DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba16 => DynamicImage::from(image.to_rgba8()),
            image::ColorType::Rgb32F => DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba32F => DynamicImage::from(image.to_rgba8()),
            _ => image,
        };

        match image.color() {
            image::ColorType::Rgb8 => {
                colorspace = gdk_pixbuf::Colorspace::Rgb;
                has_alpha = false;
                bits_per_sample = 8;
                rowstride = 3 * width;
            }
            image::ColorType::Rgba8 => {
                colorspace = gdk_pixbuf::Colorspace::Rgb;
                has_alpha = true;
                bits_per_sample = 8;
                rowstride = 4 * width;
            }
            _ => {
                return Err(format!("Unsupported color space {:?}", image.color()).into());
            }
        }
        // println!(
        //     "Image.rs {:?} {width}x{height} alpha={has_alpha}",
        //     im.color()
        // );
        let pixbuf = Pixbuf::from_bytes(
            &Bytes::from(image.as_bytes()),
            colorspace,
            has_alpha,
            bits_per_sample,
            width as i32,
            height as i32,
            rowstride as i32,
        );
        Ok(pixbuf)
    }

    pub fn pixbuf_scale(pixbuf: Pixbuf, size: i32) -> Option<Pixbuf> {
        let width = pixbuf.width();
        let height = pixbuf.height();

        let (thumb_width, thumb_height) = if width > height {
            (size, height * size / width)
        } else {
            (width * size / height, size)
        };

        pixbuf.scale_simple(thumb_width, thumb_height, gdk_pixbuf::InterpType::Bilinear)
    }
}
