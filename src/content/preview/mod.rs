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

use std::{
    fs::read,
    path::{Path, PathBuf},
};

use base64::{engine::general_purpose, Engine};

use crate::{
    classification::FileFormat,
    content::Content,
    error::MviewResult,
    image::{Color, TextCanvas, TransparencyMode, ZoomMode},
    rect::PointD,
    util::mview_hash,
};

pub struct Preview {
    file_format: FileFormat,
    path: PathBuf,
    preview: PathBuf,
}

fn get_resource_xml(resource_path: &str) -> Option<String> {
    // 1. Look up the resource data globally
    // gio::ResourceLookupFlags::NONE is standard for basic retrieval
    if let Ok(bytes) = gio::resources_lookup_data(
        &format!("/icons/scalable/apps/{resource_path}"),
        gio::ResourceLookupFlags::NONE,
    ) {
        // 2. Convert glib::Bytes to a UTF-8 String
        // return Some(String::from_utf8_lossy(&bytes).to_string());
        let inner_xml = String::from_utf8_lossy(&bytes);

        // 2. Encode to Base64
        let b64_svg = general_purpose::STANDARD.encode(inner_xml.as_bytes());
        let data_uri = format!("data:image/svg+xml;base64,{}", b64_svg);
        Some(data_uri)
    } else {
        None
    }
}

impl Preview {
    pub fn new(file_format: FileFormat, path: &Path) -> Self {
        Self {
            file_format,
            path: path.into(),
            preview: mview_hash(path, None, "mprev"),
        }
    }

    pub fn content(&self) -> MviewResult<Content> {
        dbg!(&self.preview);
        if self.preview.exists() {
            self.preview_content()
        } else {
            self.icon_content()
        }
    }

    pub fn preview_content(&self) -> MviewResult<Content> {
        let mut sheet = TextCanvas::new_auto(); // (800, 800, FONT_SIZE);
        sheet.header(&self.path);
        let content_area = sheet.content_area();
        let img = read(&self.preview)?;
        let canvas = sheet.canvas();

        let grid = 4;
        let dx = content_area.width() / grid as f64;
        let dy = content_area.height() / grid as f64;
        for y in 0..grid {
            for x in 0..grid {
                canvas.add_image_bytes(
                    content_area.point0() + PointD::new(x as f64 * dx, y as f64 * dy),
                    Some(dx),
                    Some(dy),
                    "image/jpeg",
                    &img,
                );
            }
        }

        sheet.show_open_text();

        Ok(Content::new_svg(
            sheet.into_svg_tree()?,
            None,
            ZoomMode::NotSpecified,
            TransparencyMode::Black,
        ))
    }

    pub fn icon_content(&self) -> MviewResult<Content> {
        let mut sheet = TextCanvas::new_auto(); // (800, 800, FONT_SIZE);
        sheet.header(&self.path);

        if let Some(icon) = match self.file_format {
            FileFormat::Image(_) => get_resource_xml("mv6-image.svg"),
            FileFormat::Archive(_) => get_resource_xml("mv6-box.svg"),
            FileFormat::Document(_) => get_resource_xml("mv6-doc.svg"),
            FileFormat::Video(_) => get_resource_xml("mv6-video.svg"),
            FileFormat::Folder => get_resource_xml("mv6-folder.svg"),
            FileFormat::Unknown => get_resource_xml("mv6-unknown.svg"),
        } {
            let canvas = sheet.canvas();
            canvas.add_image(
                PointD::new(canvas.width() as f64 / 2.0 - 100.0, 150.0),
                Some(200.0),
                None,
                icon,
            );
            canvas.add_message(
                PointD::new(canvas.width() as f64 / 2.0, 460.0),
                &self.file_format.to_string(),
                Color::Glaucous,
            );
        }

        sheet.show_open_text();

        Ok(Content::new_svg(
            sheet.into_svg_tree()?,
            None,
            ZoomMode::NotSpecified,
            TransparencyMode::Black,
        ))
    }
}
