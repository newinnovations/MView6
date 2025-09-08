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

#![allow(dead_code)]

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{offset::LocalResult, Local, TimeZone};
use human_bytes::human_bytes;
use resvg::usvg::Tree;
use syntect::{easy::HighlightLines, highlighting::Style};

use crate::{
    category::Category,
    config::config,
    error::MviewResult,
    file_view::{model::Row, Direction},
    image::{
        colors::{Color, MViewColor},
        svg::{
            creator::{FontWeight, LineStyle},
            text_sheet::{svg_options, TextSheet},
        },
    },
    profile::performance::Performance,
    rect::{RectD, SizeD, VectorD},
    util::{ellipsis_middle, path_to_directory, path_to_extension, path_to_filename},
};

const FONT_SIZE_TITLE: u32 = 24;
const FONT_SIZE: u32 = 14;
const LINES_PER_PAGE: usize = 32;

const BYTES_PER_LINE: usize = 16;
const WIDTH_ADDRESS: f64 = 6.5;
const WIDTH_HEX: f64 = 2.0;
const WIDTH_ASCII: f64 = 5.4;

const MAX_LINE_LENGTH: usize = 142;

pub struct RawContent {
    pub path: PathBuf,
    pub data: Arc<Vec<u8>>,
}

impl RawContent {
    pub fn size(&self) -> SizeD {
        SizeD::new(800.0, 800.0)
    }

    pub fn num_pages(&self) -> usize {
        1 + (self.data.len().saturating_sub(1) / (LINES_PER_PAGE * BYTES_PER_LINE))
    }

    pub fn prepare(&self, page: usize) -> MviewResult<Tree> {
        let mut sheet = TextSheet::new(800, 800, FONT_SIZE);
        sheet.add_line(
            &path_to_directory(&self.path),
            sheet
                .base_style()
                .font_family("Liberation Sans")
                .color(Color::FolderTitle),
        );
        sheet.delta_y(0.5);
        sheet.add_line(
            &path_to_filename(&self.path),
            sheet
                .base_style()
                .font_size(FONT_SIZE_TITLE)
                .color(Color::Yellow)
                .font_weight(FontWeight::Bold),
        );
        sheet.delta_y(0.8);

        let start_line = page * LINES_PER_PAGE;
        let total_lines = self.data.len().div_ceil(BYTES_PER_LINE);
        for line in start_line..total_lines.min(start_line + LINES_PER_PAGE) {
            self.draw_line(&mut sheet, line * BYTES_PER_LINE);
        }

        sheet.show_page_no(page, self.num_pages());
        let svg_content = sheet.finish().render();
        Ok(Tree::from_str(&svg_content, &svg_options())?)
    }

    fn draw_line(&self, sheet: &mut TextSheet, offset: usize) {
        sheet.delta_y(1.5);

        let line_start = sheet.pos();

        let end_offset = (offset + BYTES_PER_LINE).min(self.data.len());
        let line_data = &self.data[offset..end_offset];

        sheet.add_fragment(&format!("{:08x}", offset), sheet.base_style());

        sheet.delta_x(WIDTH_ADDRESS);

        let hex_start = sheet.pos();

        for (i, &byte) in line_data.iter().enumerate() {
            sheet.add_fragment(
                &format!("{:02x}", byte),
                sheet.base_style().color(Color::White),
            );
            sheet.delta_x(WIDTH_HEX);
            if i % 8 == 7 {
                sheet.delta_x(WIDTH_HEX / 2.0);
            }
        }

        sheet.set_pos(hex_start + sheet.base_style().delta_x(WIDTH_HEX * 17.0));

        sheet.add_fragment("|", sheet.base_style());
        sheet.delta_x(WIDTH_HEX / 2.0);

        let (data1, data2) = Self::split_bytes(line_data);
        Self::ascii(sheet, data1);
        sheet.delta_x(WIDTH_ASCII);
        if !data2.is_empty() {
            Self::ascii(sheet, data2);
        }
        sheet.delta_x(WIDTH_ASCII);

        sheet.add_fragment("|", sheet.base_style());

        sheet.set_pos(line_start);
    }

    fn ascii(sheet: &mut TextSheet, data: &[u8]) {
        let ascii_string: String = data
            .iter()
            .map(|&b| {
                if (32..=126).contains(&b) {
                    b as char
                } else {
                    '.'
                }
            })
            .collect();
        sheet.add_fragment(&ascii_string, sheet.base_style().color(Color::Cyan));
    }

    fn split_bytes(data: &[u8]) -> (&[u8], &[u8]) {
        data.split_at(data.len().min(8))
    }
}

pub struct TextContent {
    pub path: PathBuf,
    pub syntax_ext: String,
    pub text: Arc<Vec<String>>,
}

impl TextContent {
    pub fn new<P: AsRef<Path>>(path: P, text: Vec<String>) -> Self {
        let extension = path_to_extension(&path);
        let syntax_ext = match config().ps.find_syntax_by_extension(&extension) {
            Some(_) => extension,
            None => "txt".to_string(),
        };
        Self {
            path: path.as_ref().into(),
            text: text.into(),
            syntax_ext,
        }
    }

    pub fn size(&self) -> SizeD {
        SizeD::new(1200.0, 800.0)
    }

    pub fn num_pages(&self) -> usize {
        1 + (self.text.len().saturating_sub(1) / LINES_PER_PAGE)
    }

    pub fn prepare(&self, page: usize) -> MviewResult<Tree> {
        let syntax = config()
            .ps
            .find_syntax_by_extension(&self.syntax_ext)
            .unwrap();
        let theme = config().ts.themes.get("base16-mocha.dark").unwrap();
        let mut h = HighlightLines::new(syntax, theme);
        let mut sheet = TextSheet::new(1200, 800, FONT_SIZE);
        sheet.add_line(
            &path_to_directory(&self.path),
            sheet
                .base_style()
                .font_family("Liberation Sans")
                .color(Color::FolderTitle),
        );
        sheet.delta_y(0.5);
        sheet.add_line(
            &path_to_filename(&self.path),
            sheet
                .base_style()
                .font_size(FONT_SIZE_TITLE)
                .color(Color::Yellow)
                .font_weight(FontWeight::Bold),
        );
        sheet.delta_y(0.8);

        let ps = &config().ps;
        for line in self
            .text
            .as_ref()
            .iter()
            .skip(page * LINES_PER_PAGE)
            .take(LINES_PER_PAGE)
        {
            let line = limit_string(line);
            let ranges: Vec<(Style, &str)> = h.highlight_line(&line, ps).unwrap();
            sheet.delta_y(1.5);
            let spans = ranges
                .iter()
                .map(|(style, text)| (*text, style.foreground.into()))
                .collect();
            sheet.add_mulit_color_fragment(spans, sheet.base_style());
        }

        sheet.show_page_no(page, self.num_pages());
        let svg_content = sheet.finish().render();
        Ok(Tree::from_str(&svg_content, &svg_options())?)
    }
}

impl From<syntect::highlighting::Color> for MViewColor {
    fn from(c: syntect::highlighting::Color) -> Self {
        MViewColor::new(c.r, c.g, c.b)
    }
}

fn limit_string(s: &str) -> String {
    if s.chars().count() <= MAX_LINE_LENGTH {
        s.to_string()
    } else {
        s.chars().take(MAX_LINE_LENGTH).collect()
    }
}

pub struct ListContent {
    pub path: PathBuf,
    pub list: Arc<Vec<Row>>,
}

impl ListContent {
    pub fn new(path: PathBuf, list: Vec<Row>) -> Self {
        Self {
            path,
            list: list.into(),
        }
    }

    pub fn size(&self) -> SizeD {
        SizeD::new(800.0, 800.0)
    }

    pub fn num_pages(&self) -> usize {
        1 + (self.list.len().saturating_sub(1) / LINES_PER_PAGE)
    }

    pub fn prepare(&self, page: usize) -> MviewResult<Tree> {
        let mut sheet = TextSheet::new(800, 800, FONT_SIZE);
        sheet.add_line(
            &path_to_directory(&self.path),
            sheet
                .base_style()
                .font_family("Liberation Sans")
                .color(Color::FolderTitle),
        );
        sheet.delta_y(0.5);
        sheet.add_line(
            &path_to_filename(&self.path),
            sheet
                .base_style()
                .font_size(FONT_SIZE_TITLE)
                .color(Color::Yellow)
                .font_weight(FontWeight::Bold),
        );
        sheet.delta_y(0.8);
        for row in self
            .list
            .iter()
            .skip(page * LINES_PER_PAGE)
            .take(LINES_PER_PAGE)
        {
            dbg!(sheet.pos());
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
            let name = ellipsis_middle(&row.name, 59);
            let line = format!("{cat_text} {modified_text:<19} {size_text:>10} {}", name);
            // 3+1+19+1+10+1+59=94
            sheet.add_line(&line, sheet.base_style().color(colors.1));
        }
        sheet.show_page_no(page, self.num_pages());

        sheet.add_grid(
            RectD::new(30.0, 70.2, 800.0, 750.0),
            VectorD::new(8.2, 10.5), // 21.0),
            LineStyle::new().stroke(Color::Olive).stroke_width(0.3),
        );

        let svg_content = sheet.finish().render();
        Ok(Tree::from_str(&svg_content, &svg_options())?)
    }
}

pub enum PaginatedContentData {
    Raw(RawContent),
    Text(TextContent),
    List(ListContent),
}

pub struct PaginatedContent {
    pub data: PaginatedContentData,
    pub page: usize,
    pub rendered: Option<Arc<Tree>>,
}

impl PaginatedContent {
    pub fn new_text<P: AsRef<Path>>(path: P, lines: Vec<String>) -> Self {
        Self {
            data: PaginatedContentData::Text(TextContent::new(path, lines)),
            page: 0,
            rendered: None,
        }
    }

    pub fn new_raw<P: AsRef<Path>>(path: P, buffer: Vec<u8>) -> Self {
        Self {
            data: PaginatedContentData::Raw(RawContent {
                path: path.as_ref().into(),
                data: buffer.into(),
            }),
            page: 0,
            rendered: None,
        }
    }

    pub fn new_list<P: AsRef<Path>>(path: P, list: Vec<Row>) -> Self {
        Self {
            data: PaginatedContentData::List(ListContent {
                path: path.as_ref().into(),
                list: list.into(),
            }),
            page: 0,
            rendered: None,
        }
    }

    pub fn size(&self) -> SizeD {
        match &self.rendered {
            Some(tree) => {
                let size = tree.size();
                SizeD::new(size.width().into(), size.height().into())
            }
            None => SizeD::default(),
        }
    }

    pub fn prepare(&mut self) {
        let duration = Performance::start();
        self.rendered = match &self.data {
            PaginatedContentData::Raw(content) => content.prepare(self.page),
            PaginatedContentData::Text(content) => content.prepare(self.page),
            PaginatedContentData::List(content) => content.prepare(self.page),
        }
        .ok()
        .map(Arc::new);
        duration.elapsed("svg parse");
    }

    pub fn num_pages(&self) -> usize {
        match &self.data {
            PaginatedContentData::Raw(content) => content.num_pages(),
            PaginatedContentData::Text(content) => content.num_pages(),
            PaginatedContentData::List(content) => content.num_pages(),
        }
    }

    /// Here we handle the actual page navigation, returns `true` if we navigated to a new
    /// page, `false` if we exhausted the number of pages.
    pub fn navigate_page(&mut self, direction: Direction, count: usize) -> bool {
        match direction {
            Direction::Up => {
                if self.page >= count {
                    self.page -= count;
                    self.prepare();
                    true
                } else {
                    false
                }
            }
            Direction::Down => {
                let num_pages = self.num_pages();
                if self.page + count < num_pages {
                    self.page += count;
                    self.prepare();
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn has_alpha(&self) -> bool {
        false
    }
}
