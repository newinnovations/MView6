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

#[cfg(not(windows))]
use std::path::Path;

use resvg::usvg::{fontdb, Options, Tree};

use crate::{
    content::Content,
    error::MviewResult,
    image::{
        colors::{Color, MViewColor},
        svg::creator::{SvgCanvas, TextAnchor, TextStyle},
        view::{data::TransparencyMode, ZoomMode},
    },
    rect::PointD,
};

const FONT_FAMILY: &str = "Cascadia Mono";

pub struct TextSheet {
    canvas: SvgCanvas,
    style: TextStyle,
    pos: PointD,
}

impl TextSheet {
    pub fn new(width: u32, height: u32, font_size: u32) -> Self {
        Self {
            canvas: SvgCanvas::new(width, height).background(Color::Black),
            style: TextStyle::new()
                .font_family(FONT_FAMILY)
                .font_size(font_size)
                .color(Color::DarkGray)
                .anchor(TextAnchor::Start),
            pos: PointD::new(30.0, 10.0),
        }
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
                PointD::new(30.0, self.canvas.height() as f64 - 20.0),
                &format!("Page {} of {total}", page + 1),
                style,
            );
        }
    }

    pub fn finish(mut self) -> SvgCanvas {
        self.canvas.add_watermark(PointD::new(
            self.canvas.width() as f64 - 20.0,
            self.canvas.height() as f64 - 20.0,
        ));
        self.canvas
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
