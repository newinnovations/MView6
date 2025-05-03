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

pub mod gdk;
pub mod image_rs;
pub mod webp;

use crate::{category::Category, image::Image, profile::performance::Performance};
use exif::Exif;
use gdk::GdkImageLoader;
use image::DynamicImage;
use image_rs::RsImageLoader;
use std::{
    fs,
    io::{BufRead, BufReader, Cursor, Seek},
    path::Path,
};

use super::{
    draw::{draw_error, draw_text},
    view::ZoomMode,
};

pub struct ImageLoader {}

impl ImageLoader {
    pub fn image_from_file(filename: &str) -> Image {
        let duration = Performance::start();

        let path = Path::new(&filename);

        let cat = match fs::metadata(path) {
            Ok(metadata) => Category::determine(filename, metadata.is_dir()),
            Err(_) => Category::Unsupported,
        };

        match cat {
            Category::Folder | Category::Archive | Category::Unsupported => {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default();
                return draw_text(&cat.name(), name, cat.colors());
            }
            _ => (),
        };

        if filename.to_lowercase().contains(".svg") {
            if let Ok(Some(handle)) = rsvg::Handle::from_file(filename) {
                duration.elapsed("decode svg (file)");
                return Image::new_svg(handle, None, ZoomMode::NotSpecified);
            }
        }

        let input = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(error) => return draw_error(error.into()),
        };
        let mut reader = BufReader::new(input);

        let image = if let Ok(im) = GdkImageLoader::image_from_reader(&mut reader) {
            im
        } else {
            match RsImageLoader::image_from_file(reader) {
                Ok(im) => im,
                Err(e) => draw_error(e),
            }
        };

        duration.elapsed("decode (file)");

        image
    }

    pub fn image_from_memory(buf: Vec<u8>, try_svg: bool) -> Image {
        let duration = Performance::start();

        if try_svg {
            if let Ok(Some(handle)) = rsvg::Handle::from_data(&buf) {
                duration.elapsed("decode svg (mem)");
                return Image::new_svg(handle, None, ZoomMode::NotSpecified);
            }
        }

        let mut reader = Cursor::new(buf);

        let image = if let Ok(im) = GdkImageLoader::image_from_reader(&mut reader) {
            im
        } else {
            match RsImageLoader::image_from_memory(reader) {
                Ok(im) => im,
                Err(e) => draw_error(e),
            }
        };

        duration.elapsed("decode (mem)");

        image
    }

    pub fn image_from_svg_data(buf: &[u8], tag: Option<String>) -> Option<Image> {
        if let Ok(Some(handle)) = rsvg::Handle::from_data(buf) {
            Some(Image::new_svg(handle, tag, ZoomMode::Fill))
        } else {
            None
        }
    }
}

pub struct ImageSaver {}

impl ImageSaver {
    pub fn save_thumbnail(base_directory: &str, filename: &str, image: &DynamicImage) {
        let thumbnail_dir = format!("{}/.mview", base_directory);
        if !Path::new(&thumbnail_dir).exists() {
            if let Err(error) = fs::create_dir(&thumbnail_dir) {
                println!("Failed to create thumbnail directory: {:?}", error);
                return;
            }
        }
        let thumbnail_path = format!("{thumbnail_dir}/{filename}");

        let image = match image.color() {
            image::ColorType::L16 => &DynamicImage::from(image.to_luma8()),
            image::ColorType::La16 => &DynamicImage::from(image.to_luma_alpha8()),
            image::ColorType::Rgb16 => &DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba16 => &DynamicImage::from(image.to_rgba8()),
            image::ColorType::Rgb32F => &DynamicImage::from(image.to_rgb8()),
            image::ColorType::Rgba32F => &DynamicImage::from(image.to_rgba8()),
            _ => image,
        };

        let format = match image.color() {
            image::ColorType::L8 => image::ImageFormat::Jpeg,
            image::ColorType::La8 => image::ImageFormat::WebP,
            image::ColorType::Rgb8 => image::ImageFormat::Jpeg,
            image::ColorType::Rgba8 => image::ImageFormat::WebP,
            _ => {
                println!(
                    "Unsupported image colortype when writing thumbnail {:?}",
                    image.color()
                );
                return;
            }
        };

        if let Err(error) = image.save_with_format(thumbnail_path, format) {
            println!("Failed to write thumbnail: {:?}", error);
        }
    }
}

pub trait ExifReader {
    fn exif(&mut self) -> Option<Exif>;
}

impl<T: BufRead + Seek> ExifReader for T {
    fn exif(&mut self) -> Option<Exif> {
        let duration = Performance::start();
        self.rewind().ok()?;
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(self);
        self.rewind().ok()?;
        duration.elapsed("exif");
        exif.ok()
    }
}
