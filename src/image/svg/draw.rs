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

use chrono::{offset::LocalResult, Local, TimeZone};
use human_bytes::human_bytes;
use resvg::usvg::{fontdb, Options, Tree};
use std::{fs::File, io::Read, path::Path};
use syntect::parsing::SyntaxReference;

use crate::{
    category::Category,
    content::Content,
    error::MviewResult,
    file_view::model::Row,
    image::{
        colors::Color,
        svg::{
            creator::{FontWeight, SvgCanvas},
            hexview::HexdumpViewer,
            highlight::TextHighLighter,
            text_sheet::TextSheet,
        },
        view::{data::TransparencyMode, ZoomMode},
    },
    util::{path_to_directory, path_to_filename, read_lines_with_limits},
};

const FONT_SIZE_TITLE: u32 = 24;
const FONT_SIZE: u32 = 14;

fn svg_options<'a>() -> Options<'a> {
    let mut fontdb = fontdb::Database::new();
    load_font_file(&mut fontdb, "LiberationSans-Regular.ttf");
    load_font_file(&mut fontdb, "LiberationSans-Bold.ttf");
    load_font_file(&mut fontdb, "CascadiaMono-Regular.ttf");
    Options::<'_> {
        fontdb: fontdb.into(),
        ..Default::default()
    }
}

fn load_font_file(fontdb: &mut fontdb::Database, name: &str) {
    let path = {
        #[cfg(windows)]
        {
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()));
            match exe_dir {
                Some(exe_dir) => exe_dir.join(name),
                None => {
                    eprintln!("Failed to obtain directory of executable");
                    return;
                }
            }
        }
        #[cfg(not(windows))]
        Path::new("/usr/lib/mview6").join(name)
    };
    if fontdb.load_font_file(&path).is_err() {
        eprintln!("Failed to load font {path:?}");
    }
}

pub fn svg_text_sheet(
    title: &str,
    msg: &str,
    colors: (Color, Color, Color),
) -> MviewResult<Content> {
    let svg_content = SvgCanvas::create_text_sheet(title, msg, colors);
    let tree = Tree::from_str(&svg_content, &svg_options())?;
    Ok(Content::new_svg(
        tree,
        None,
        ZoomMode::NotSpecified,
        TransparencyMode::Black,
    ))
}

pub fn svg_hexdump(path: &Path) -> MviewResult<Content> {
    let file = File::open(path)?;
    let mut buffer = Vec::new();
    file.take(1024).read_to_end(&mut buffer)?;
    let mut hexview = HexdumpViewer::new(path, buffer);
    hexview.draw();
    let svg_content = hexview.finish().render();
    let tree = Tree::from_str(&svg_content, &svg_options())?;
    Ok(Content::new_svg(
        tree,
        None,
        ZoomMode::NotSpecified,
        TransparencyMode::Black,
    ))
}

pub fn svg_highlight(path: &Path, syntax: &SyntaxReference) -> MviewResult<Content> {
    let lines = read_lines_with_limits(path, Some(1000), Some(16384))?;
    let mut hexview = TextHighLighter::new(path, syntax);
    hexview.draw(&lines);
    let svg_content = hexview.finish().render();
    let tree = Tree::from_str(&svg_content, &svg_options())?;
    Ok(Content::new_svg(
        tree,
        None,
        ZoomMode::NotSpecified,
        TransparencyMode::Black,
    ))
}

pub fn svg_directory_list(path: &Path, store: &[Row]) -> MviewResult<Content> {
    let mut sheet = TextSheet::new(800, 800, FONT_SIZE);
    sheet.add_line(
        &path_to_directory(path),
        sheet
            .base_style()
            .font_family("Liberation Sans")
            .color(Color::FolderTitle),
    );
    sheet.delta_y(0.5);
    sheet.add_line(
        &path_to_filename(path),
        sheet
            .base_style()
            .font_size(FONT_SIZE_TITLE)
            .color(Color::Yellow)
            .font_weight(FontWeight::Bold),
    );
    sheet.delta_y(0.8);
    for row in store.iter().take(32) {
        let modified_text = if row.modified > 0 {
            if let LocalResult::Single(dt) = Local.timestamp_opt(row.modified as i64, 0) {
                dt.format("%d-%m-%Y %H:%M:%S").to_string()
            } else {
                String::default()
            }
        } else {
            String::default()
        };
        let size_text = if row.size > 0 {
            human_bytes(row.size as f64)
        } else {
            String::default()
        };
        let cat = Category::from(row.category);
        let cat_text = cat.short();
        let colors = cat.colors();
        let line = format!(
            "{cat_text} {modified_text:<19} {size_text:>10} {}",
            row.name
        );
        sheet.add_line(&line, sheet.base_style().color(colors.1));
    }
    let svg_content = sheet.finish().render();
    let tree = Tree::from_str(&svg_content, &svg_options())?;
    Ok(Content::new_svg(
        tree,
        None,
        ZoomMode::NotSpecified,
        TransparencyMode::Black,
    ))
}
