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

use crate::{
    backends::{filesystem::FileSystem, Backend, MarArchive, RarArchive, ZipArchive},
    category::Category,
    content::{content_type::ContentType, paginated::PaginatedContent, Content},
    error::MviewResult,
    file_view::model::BackendRef,
    image::{
        draw::{draw_error, draw_text},
        provider::{gdk::GdkImageLoader, image_rs::RsImageLoader, internal::InternalImageLoader},
        view::{data::TransparencyMode, ZoomMode},
    },
    profile::performance::Performance,
    util::path_to_extension,
};
use resvg::usvg::{self, fontdb::Database, Options, Tree};
use std::{
    fs,
    io::{BufReader, Cursor, Read, Seek},
    path::Path,
};

pub const MAX_CONTENT_SIZE: u64 = 1024 * 1024;

pub struct ContentLoader {}

impl ContentLoader {
    /// Load content from file
    ///
    /// Called by the bookmarks and filesystem backends
    ///
    /// 1. Determine file or directory
    /// 2. Examine extension
    ///    - if known (internal) extension handle accordingly
    ///    - if known by syntax highligher handle there
    /// 3. Read (part of) file into memory
    ///    - if known (internal) format recognized handle accordingly
    ///    - if textual content handle by highlighter with "txt" format
    ///    - handle raw
    pub fn content_from_file(path: &Path) -> Content {
        if path.is_dir() {
            let list = FileSystem::new(path).list().clone();
            return Content::new_list(path, BackendRef::FileSystem(path.into()), list);
        }

        let ext = path_to_extension(path);
        let content_type = ContentType::from_extension(&ext);
        // dbg!(content_type);
        if content_type != ContentType::Unknown {
            return Self::load_file(content_type, path);
        }

        let data = match Self::read_file(path) {
            Ok(data) => data,
            Err(e) => return draw_error(path, e),
        };

        let content_type = ContentType::determine(&data);
        if content_type != ContentType::Unknown {
            return Self::load_file(content_type, path);
        }

        // is it text? FIXME: handle utf16
        Content::new_paginated(if data.contains(&0) {
            PaginatedContent::new_raw(path, data)
        } else {
            match str::from_utf8(&data) {
                Ok(text) => {
                    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
                    // if lines.iter().any(|line| line.len() > 200) {
                    //     PaginatedContent::new_raw(path, data)
                    // } else {
                    PaginatedContent::new_text(path, lines)
                    // }
                }
                Err(_) => PaginatedContent::new_raw(path, data),
            }
        })
    }

    fn load_file(content_type: ContentType, path: &Path) -> Content {
        match content_type {
            ContentType::Epub | ContentType::Pdf => {
                // draw_text("Document", "PDF/EPUB", Category::Document.colors())
                Content::new_preview(path, BackendRef::Pdfium(path.into()))
            }
            ContentType::Mar => {
                let list = MarArchive::new(path).list().clone();
                Content::new_list(path, BackendRef::MarArchive(path.into()), list)
            }
            ContentType::Rar => {
                let list = RarArchive::new(path).list().clone();
                Content::new_list(path, BackendRef::RarArchive(path.into()), list)
            }
            ContentType::Zip => {
                let list = ZipArchive::new(path).list().clone();
                Content::new_list(path, BackendRef::ZipArchive(path.into()), list)
            }
            ContentType::Svg => match Self::read_svg(path) {
                Ok(tree) => Content::new_svg(
                    tree,
                    None,
                    ZoomMode::NotSpecified,
                    TransparencyMode::NotSpecified,
                ),
                Err(error) => draw_error(path, error),
            },
            ContentType::Avif
            | ContentType::Gif
            | ContentType::Heic
            | ContentType::Jpeg
            | ContentType::Pcx
            | ContentType::Png
            | ContentType::Webp => {
                let input = match std::fs::File::open(path) {
                    Ok(file) => file,
                    Err(error) => return draw_error(path, error.into()),
                };
                let mut reader = BufReader::new(input);

                if let Ok(im) = GdkImageLoader::image_from_reader(&mut reader) {
                    im
                } else {
                    let _ = reader.rewind();
                    if let Ok(im) = InternalImageLoader::image_from_reader(&mut reader) {
                        im
                    } else {
                        let _ = reader.rewind();
                        match RsImageLoader::image_from_file(reader) {
                            Ok(im) => im,
                            Err(e) => draw_error(path, e),
                        }
                    }
                }
            }
            ContentType::Unknown => draw_text(
                "Unknown",
                "Content not recognized",
                Category::Unsupported.colors(),
            ),
        }
    }

    /// Load content from file
    ///
    /// Called by the zip and rar backends
    pub fn content_from_memory(buf: Vec<u8>, path: &Path) -> Content {
        let duration = Performance::start();

        if buf.starts_with(&[0x3c, 0x3f]) || buf.starts_with(&[0x1f, 0x8b]) {
            let svg_options = usvg::Options::default();
            if let Ok(tree) = Tree::from_data(&buf, &svg_options) {
                duration.elapsed("decode svg (mem)");
                return Content::new_svg(
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
                    Err(e) => draw_error(path, e),
                }
            }
        };

        duration.elapsed("decode (mem)");

        image
    }

    pub fn content_from_svg_data(buf: &[u8], tag: Option<String>) -> Option<Content> {
        let svg_options = usvg::Options::default();
        if let Ok(tree) = Tree::from_data(buf, &svg_options) {
            Some(Content::new_svg(
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

    fn read_file<P: AsRef<Path>>(path: P) -> MviewResult<Vec<u8>> {
        let file = std::fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.take(MAX_CONTENT_SIZE).read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
