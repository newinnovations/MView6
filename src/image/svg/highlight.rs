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

use std::path::Path;

use syntect::{easy::HighlightLines, highlighting::Style, parsing::SyntaxReference};

use crate::{
    config::config,
    image::{
        colors::{Color, MViewColor},
        svg::{
            creator::{FontWeight, SvgCanvas},
            text_sheet::TextSheet,
        },
    },
    util::{path_to_directory, path_to_filename},
};

const FONT_SIZE_TITLE: u32 = 24;
const FONT_SIZE: u32 = 14;
const MAX_LINE_LENGTH: usize = 142;

pub struct TextHighLighter<'a> {
    sheet: TextSheet,
    h: HighlightLines<'a>,
}

impl<'a> TextHighLighter<'a> {
    pub fn new(path: &Path, syntax: &SyntaxReference) -> Self {
        let theme = config().ts.themes.get("base16-mocha.dark").unwrap();
        let h = HighlightLines::new(syntax, theme);
        let mut sheet = TextSheet::new(1200, 800, FONT_SIZE);
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
        Self { sheet, h }
    }

    pub fn draw(&mut self, lines: &Vec<String>) {
        let lines_visible = 32;
        let mut line_no: i32 = 0;
        let ps = &config().ps;
        for line in lines {
            let line = limit_string(line);
            let ranges: Vec<(Style, &str)> = self.h.highlight_line(&line, ps).unwrap();
            // Print the highlighted line to the terminal
            // syntect::util::as_24_bit_terminal_escaped(&mut handle, &ranges[..], true);
            // print!("{line}");
            // dbg!(ranges);
            self.draw_line(ranges);
            line_no += 1;
            if line_no >= lines_visible {
                break;
            }
            // let escaped = as_24_bit_terminal_escaped(&ranges[..], true);
            // print!("{}", escaped);
        }
    }

    fn draw_line(&mut self, ranges: Vec<(Style, &str)>) {
        self.sheet.delta_y(1.5);
        let spans = ranges
            .iter()
            .map(|(style, text)| (*text, style.foreground.into()))
            .collect();
        self.sheet
            .add_mulit_color_fragment(spans, self.sheet.base_style());
    }

    pub fn finish(self) -> SvgCanvas {
        self.sheet.finish()
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
