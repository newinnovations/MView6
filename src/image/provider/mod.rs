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

pub mod gdk;
pub mod image_rs;
pub mod internal;
pub mod surface;
pub mod webp;

use crate::{
    backends::Backend,
    category::Category,
    config::config,
    error::MviewResult,
    image::{
        svg::draw::{svg_directory_list, svg_hexdump, svg_highlight},
        view::data::TransparencyMode,
        Image,
    },
    profile::performance::Performance,
    util::{path_to_extension, path_to_filename},
};
use exif::Exif;
use gdk::GdkImageLoader;
use image::DynamicImage;
use image_rs::RsImageLoader;
use internal::InternalImageLoader;
use resvg::usvg::{self, fontdb::Database, Options, Tree};
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
    pub fn image_from_file(path: &Path) -> Image {
        let duration = Performance::start();

        // let path = Path::new(&filename);

        let cat = match fs::metadata(path) {
            Ok(metadata) => Category::determine(path, metadata.is_dir()),
            Err(_) => Category::Unsupported,
        };

        let name = path_to_filename(path);
        match cat {
            Category::Unsupported => {
                let syntax = config()
                    .ps
                    .find_syntax_by_extension(&path_to_extension(path));
                if let Ok(image) = match syntax {
                    Some(syntax) => svg_highlight(path, syntax),
                    None => svg_hexdump(path),
                } {
                    return image;
                } else {
                    return draw_text(&cat.name(), &name, cat.colors());
                }
            }
            Category::Folder | Category::Archive => {
                let backend = <dyn Backend>::new(path);
                let store = backend.store();
                if let Ok(image) = svg_directory_list(path, store) {
                    return image;
                } else {
                    return draw_text(&cat.name(), &name, cat.colors());
                }
            }
            Category::Document => {
                return draw_text(&cat.name(), &name, cat.colors());
            }
            _ => (),
        };

        let is_svg = path
            .extension()
            .map(|ext| ext.to_string_lossy().starts_with("svg"))
            .unwrap_or_default();
        if is_svg {
            if let Ok(svg) = Self::read_svg(path) {
                duration.elapsed("decode svg (file)");
                return Image::new_svg(
                    svg,
                    None,
                    ZoomMode::NotSpecified,
                    TransparencyMode::NotSpecified,
                );
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
            let _ = reader.rewind();
            if let Ok(im) = InternalImageLoader::image_from_reader(&mut reader) {
                im
            } else {
                let _ = reader.rewind();
                match RsImageLoader::image_from_file(reader) {
                    Ok(im) => im,
                    Err(e) => draw_error(e),
                }
            }
        };

        duration.elapsed("decode (file)");

        image
    }

    pub fn image_from_memory(buf: Vec<u8>) -> Image {
        let duration = Performance::start();

        if buf.starts_with(&[0x3c, 0x3f]) || buf.starts_with(&[0x1f, 0x8b]) {
            let svg_options = usvg::Options::default();
            if let Ok(tree) = Tree::from_data(&buf, &svg_options) {
                duration.elapsed("decode svg (mem)");
                return Image::new_svg(
                    tree,
                    None,
                    ZoomMode::NotSpecified,
                    TransparencyMode::NotSpecified,
                );
            }
        }

        let mut reader = Cursor::new(buf);

        let image = if let Ok(im) = GdkImageLoader::image_from_reader(&mut reader) {
            im
        } else {
            let _ = reader.rewind();
            if let Ok(im) = InternalImageLoader::image_from_reader(&mut reader) {
                im
            } else {
                let _ = reader.rewind();
                match RsImageLoader::image_from_memory(reader) {
                    Ok(im) => im,
                    Err(e) => draw_error(e),
                }
            }
        };

        duration.elapsed("decode (mem)");

        image
    }

    pub fn image_from_svg_data(buf: &[u8], tag: Option<String>) -> Option<Image> {
        let svg_options = usvg::Options::default();
        if let Ok(tree) = Tree::from_data(buf, &svg_options) {
            Some(Image::new_svg(
                tree,
                tag,
                ZoomMode::Fill,
                TransparencyMode::NotSpecified,
            ))
        } else {
            None
        }
    }

    fn read_svg(path: &Path) -> MviewResult<Tree> {
        let mut fontdb = Database::new();
        fontdb.load_system_fonts(); // This loads system fonts

        // You can also load specific fonts:
        // fontdb.load_font_file("path/to/font.ttf")?;

        // Create usvg options with the font database
        let svg_options = Options::<'_> {
            fontdb: fontdb.into(),
            ..Default::default()
        };

        let svg_data = fs::read(path)?;
        Ok(Tree::from_data(&svg_data, &svg_options)?)
    }
}

pub struct ImageSaver {}

impl ImageSaver {
    pub fn save_thumbnail(thumbnail_path: &Path, image: &DynamicImage) {
        if let Some(thumbnail_dir) = thumbnail_path.parent() {
            if !thumbnail_dir.exists() {
                if let Err(error) = fs::create_dir_all(thumbnail_dir) {
                    println!("Failed to create thumbnail directory: {error:?}");
                    return;
                }
            }
        }

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
            println!("Failed to write thumbnail: {error:?}");
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
