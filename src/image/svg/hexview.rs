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
    image::colors::Color,
    image::svg::creator::{FontWeight, SvgCanvas, TextAnchor, TextStyle},
    rect::PointD,
};

const BYTES_PER_LINE: usize = 16;
const FONT_FAMILY: &str = "Cascadia Mono";
const FONT_SIZE_TITLE: u32 = 24;
const FONT_SIZE: u32 = 14;
const WIDTH_ADDRESS: f64 = 6.5;
const WIDTH_HEX: f64 = 2.0;
const WIDTH_ASCII: f64 = 5.4;

pub struct HexdumpViewer {
    data: Vec<u8>,
    style: TextStyle,
}

impl HexdumpViewer {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            style: TextStyle::new()
                .font_family(FONT_FAMILY)
                .font_size(FONT_SIZE)
                .fill(Color::DarkGray)
                .anchor(TextAnchor::Start),
        }
    }

    pub fn draw(&self, name: &str) -> SvgCanvas {
        let mut canvas = SvgCanvas::new(800, 800).background(Color::Black);
        let mut pos = PointD::new(30.0, 10.0);
        let style = self
            .style
            .clone()
            .font_size(FONT_SIZE_TITLE)
            .fill(Color::Yellow)
            .font_weight(FontWeight::Bold);
        pos += style.delta_y(1.5);
        self.show_text(&mut canvas, pos, name, style);
        pos += self.style.delta_y(2.2);
        let lines_visible = 37;
        let total_lines = self.data.len().div_ceil(BYTES_PER_LINE);
        for line in 0..total_lines.min(lines_visible) {
            let offset = line * BYTES_PER_LINE;
            self.draw_line(&mut canvas, offset, pos);
            pos += self.style.delta_y(1.3);
        }
        canvas.add_watermark(PointD::new(780.0, 780.0));
        canvas
    }

    fn show_text(&self, canvas: &mut SvgCanvas, pos: PointD, text: &str, style: TextStyle) {
        canvas.add_text(pos, text, style);
    }

    fn draw_line(&self, canvas: &mut SvgCanvas, offset: usize, pos: PointD) {
        let mut position = pos;

        let end_offset = (offset + BYTES_PER_LINE).min(self.data.len());
        let line_data = &self.data[offset..end_offset];

        self.show_text(
            canvas,
            position,
            &format!("{:08x}", offset),
            self.style.clone(),
        );
        position += self.style.delta_x(WIDTH_ADDRESS);

        let hex_start = position;

        for (i, &byte) in line_data.iter().enumerate() {
            self.show_text(
                canvas,
                position,
                &format!("{:02x}", byte),
                self.style.clone().fill(Color::White),
            );
            position += self.style.delta_x(WIDTH_HEX);
            if i % 8 == 7 {
                position += self.style.delta_x(WIDTH_HEX / 2.0);
            }
        }

        position = hex_start + self.style.delta_x(WIDTH_HEX * 17.0);

        self.show_text(canvas, position, "|", self.style.clone());
        position += self.style.delta_x(WIDTH_HEX / 2.0);

        let (data1, data2) = Self::split_bytes(line_data);
        self.show_ascii(canvas, position, data1);
        position += self.style.delta_x(WIDTH_ASCII);
        if !data2.is_empty() {
            self.show_ascii(canvas, position, data2);
        }
        position += self.style.delta_x(WIDTH_ASCII);

        self.show_text(canvas, position, "|", self.style.clone());
    }

    fn show_ascii(&self, canvas: &mut SvgCanvas, pos: PointD, data: &[u8]) {
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
        self.show_text(
            canvas,
            pos,
            &ascii_string,
            self.style.clone().fill(Color::Cyan),
        );
    }

    fn split_bytes(data: &[u8]) -> (&[u8], &[u8]) {
        data.split_at(data.len().min(8))
    }
}
