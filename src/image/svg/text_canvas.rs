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

use resvg::usvg::{fontdb, Options};

use crate::{
    image::{
        colors::{Color, MViewColor},
        svg::canvas::{FontWeight, LineStyle, SvgCanvas, TextAnchor, TextStyle},
        view::window_size,
    },
    rect::{PointD, RectD, VectorD},
    util::{ellipsis_middle, path_to_directory, path_to_filename},
};

const FONT_FAMILY: &str = "Cascadia Mono";
const FONT_SIZE: u32 = 14;
const FONT_SIZE_TITLE: u32 = 24;
const FONT_WIDTH_TITLE: f64 = 14.065;
const OFFSET_LEFT: f64 = 30.0;
const OFFSET_RIGHT: f64 = 20.0;
const OFFSET_BOTTOM: f64 = 20.0;

pub struct TextCanvas {
    canvas: SvgCanvas,
    style: TextStyle,
    pos: PointD,
}

impl TextCanvas {
    pub fn new_auto() -> Self {
        let height = 800;
        let window_size = window_size();

        let width = if window_size.height() > 0 {
            (height as i32 * window_size.width() / window_size.height()).max(800) as u32
        } else {
            800
        };

        Self {
            canvas: SvgCanvas::new(width, height).background(Color::Black),
            style: TextStyle::new()
                .font_family(FONT_FAMILY)
                .font_size(FONT_SIZE)
                .color(Color::DarkGray)
                .anchor(TextAnchor::Start),
            pos: PointD::new(OFFSET_LEFT, 10.0),
        }

        // canvas.add_grid(
        //     RectD::new(OFFSET_LEFT, 40.0, width as f64 - OFFSET_RIGHT, 61.1),
        //     VectorD::new(FONT_WIDTH_TITLE, 21.0), // 21.0),
        //     LineStyle::new().stroke(Color::Olive).stroke_width(0.3),
        // );

        // // sheet.add_grid(
        // //     RectD::new(OFFSET_X, 76.0, 800.0, 750.0),
        // //     VectorD::new(8.2, 21.0), // 21.0),
        // //     LineStyle::new().stroke(Color::Olive).stroke_width(0.3),
        // // );

        // canvas
    }

    pub fn base_style(&self) -> TextStyle {
        self.style.clone()
    }

    pub fn add_line(&mut self, line: &str, style: TextStyle) {
        self.pos += self.style.delta_y(1.5);
        self.canvas.add_text(self.pos, line, style);
    }

    pub fn add_fragment(&mut self, fragment: &str, style: TextStyle) {
        self.canvas.add_text(self.pos, fragment, style);
    }

    pub fn add_mulit_color_fragment(&mut self, spans: Vec<(&str, MViewColor)>, style: TextStyle) {
        self.canvas.add_multicolor_text(self.pos, spans, style);
    }

    pub fn delta_x(&mut self, delta: f64) {
        self.pos += self.style.delta_x(delta);
    }

    pub fn delta_y(&mut self, delta: f64) {
        self.pos += self.style.delta_y(delta);
    }

    pub fn pos(&self) -> PointD {
        self.pos
    }

    pub fn set_pos(&mut self, pos: PointD) {
        self.pos = pos;
    }

    pub fn show_page_no(&mut self, page: usize, total: usize) {
        if total > 1 {
            let style = self.base_style().font_family("Liberation Sans");
            let font_size = style.font_size * 10 / 14;
            let style = style.font_size(font_size);
            self.canvas.add_text(
                PointD::new(OFFSET_LEFT, self.canvas.height() as f64 - 35.0),
                &format!("Page {} of {total}", page + 1),
                style,
            );
        }
    }

    pub fn show_open_text(&mut self) {
        let style = self.base_style().font_family("Liberation Sans");
        let font_size = style.font_size * 10 / 14;
        let style = style.font_size(font_size).color(Color::Glaucous);
        self.canvas.add_text(
            PointD::new(OFFSET_LEFT, self.canvas.height() as f64 - OFFSET_BOTTOM),
            "Press ENTER or double click to open",
            style,
        );
    }

    pub fn finish(mut self) -> SvgCanvas {
        self.canvas.add_watermark(PointD::new(
            self.canvas.width() as f64 - OFFSET_RIGHT,
            self.canvas.height() as f64 - OFFSET_BOTTOM,
        ));
        self.canvas
    }

    pub fn header(&mut self, path: &Path) {
        let max_len =
            (self.canvas().width() as f64 - OFFSET_LEFT - OFFSET_RIGHT) / FONT_WIDTH_TITLE;
        let max_len = max_len.floor() as usize;
        self.add_line(
            &path_to_directory(path),
            self.base_style()
                .font_family("Liberation Sans")
                .color(Color::FolderTitle),
        );
        self.delta_y(0.5);
        self.add_line(
            &ellipsis_middle(&path_to_filename(path), max_len),
            self.base_style()
                .font_size(FONT_SIZE_TITLE)
                .color(Color::Yellow)
                .font_weight(FontWeight::Bold),
        );
        self.delta_y(0.8);
    }

    pub fn canvas(&mut self) -> &mut SvgCanvas {
        &mut self.canvas
    }

    /// Add a grid to the canvas
    #[allow(dead_code)]
    pub fn add_grid(&mut self, grid: RectD, grid_size: VectorD, style: LineStyle) {
        self.canvas.add_grid(grid, grid_size, style);
    }
}

pub fn svg_options<'a>() -> Options<'a> {
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
