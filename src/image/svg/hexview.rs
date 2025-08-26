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

use crate::{
    image::{
        colors::Color,
        svg::{
            creator::{FontWeight, SvgCanvas},
            text_sheet::TextSheet,
        },
    },
    util::{path_to_directory, path_to_filename},
};

const BYTES_PER_LINE: usize = 16;
const FONT_SIZE_TITLE: u32 = 24;
const FONT_SIZE: u32 = 14;
const WIDTH_ADDRESS: f64 = 6.5;
const WIDTH_HEX: f64 = 2.0;
const WIDTH_ASCII: f64 = 5.4;

pub struct HexdumpViewer {
    data: Vec<u8>,
    sheet: TextSheet,
}

impl HexdumpViewer {
    pub fn new(path: &Path, data: Vec<u8>) -> Self {
        let mut sheet = TextSheet::new(800, 800, FONT_SIZE);
        sheet.add_line(
            &path_to_directory(path),
            sheet
                .base_style()
                .font_family("Liberation Sans")
                .fill(Color::FolderTitle),
        );
        sheet.delta_y(0.5);
        sheet.add_line(
            &path_to_filename(path),
            sheet
                .base_style()
                .font_size(FONT_SIZE_TITLE)
                .fill(Color::Yellow)
                .font_weight(FontWeight::Bold),
        );
        sheet.delta_y(0.8);
        Self { data, sheet }
    }

    pub fn draw(&mut self) {
        let lines_visible = 32;
        let total_lines = self.data.len().div_ceil(BYTES_PER_LINE);
        for line in 0..total_lines.min(lines_visible) {
            let offset = line * BYTES_PER_LINE;
            self.draw_line(offset);
        }
    }

    fn draw_line(&mut self, offset: usize) {
        self.sheet.delta_y(1.5);

        let line_start = self.sheet.pos();

        let end_offset = (offset + BYTES_PER_LINE).min(self.data.len());
        let line_data = &self.data[offset..end_offset];

        self.sheet
            .add_fragment(&format!("{:08x}", offset), self.sheet.base_style());

        self.sheet.delta_x(WIDTH_ADDRESS);

        let hex_start = self.sheet.pos();

        for (i, &byte) in line_data.iter().enumerate() {
            self.sheet.add_fragment(
                &format!("{:02x}", byte),
                self.sheet.base_style().fill(Color::White),
            );
            self.sheet.delta_x(WIDTH_HEX);
            if i % 8 == 7 {
                self.sheet.delta_x(WIDTH_HEX / 2.0);
            }
        }

        self.sheet
            .set_pos(hex_start + self.sheet.base_style().delta_x(WIDTH_HEX * 17.0));

        self.sheet.add_fragment("|", self.sheet.base_style());
        self.sheet.delta_x(WIDTH_HEX / 2.0);

        let (data1, data2) = Self::split_bytes(line_data);
        Self::ascii(&mut self.sheet, data1);
        self.sheet.delta_x(WIDTH_ASCII);
        if !data2.is_empty() {
            Self::ascii(&mut self.sheet, data2);
        }
        self.sheet.delta_x(WIDTH_ASCII);

        self.sheet.add_fragment("|", self.sheet.base_style());

        self.sheet.set_pos(line_start);
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
        sheet.add_fragment(&ascii_string, sheet.base_style().fill(Color::Cyan));
    }

    fn split_bytes(data: &[u8]) -> (&[u8], &[u8]) {
        data.split_at(data.len().min(8))
    }

    pub fn finish(self) -> SvgCanvas {
        self.sheet.finish()
    }
}
