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

use std::path::{Path, PathBuf};

use base64::{engine::general_purpose, Engine};
use resvg::usvg::Tree;

use crate::{
    classification::file_formats::FileFormat,
    content::Content,
    error::MviewResult,
    image::{
        colors::Color,
        svg::text_canvas::{svg_options, TextCanvas},
        view::{data::TransparencyMode, ZoomMode},
    },
    rect::PointD,
};

pub struct Preview {
    file_format: FileFormat,
    path: PathBuf,
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
        }
    }

    pub fn content(&self) -> MviewResult<Content> {
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

        let svg_content = sheet.finish().to_svg_string();

        let tree = Tree::from_str(&svg_content, &svg_options())?;

        Ok(Content::new_svg(
            tree,
            None,
            ZoomMode::NotSpecified,
            TransparencyMode::Black,
        ))
    }
}
