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
    image::svg::creator::{SvgCanvas, TextAnchor, TextStyle},
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
                .fill(Color::DarkGray)
                .anchor(TextAnchor::Start),
            pos: PointD::new(30.0, 10.0),
        }
    }

    pub fn base_style(&self) -> TextStyle {
        self.style.clone()
    }

    pub fn add_line(&mut self, line: &str, style: TextStyle) {
        self.pos += self.style.delta_y(1.5);
        Self::show_text(&mut self.canvas, self.pos, line, style);
    }

    pub fn add_fragment(&mut self, fragment: &str, style: TextStyle) {
        Self::show_text(&mut self.canvas, self.pos, fragment, style);
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

    fn show_text(canvas: &mut SvgCanvas, pos: PointD, text: &str, style: TextStyle) {
        canvas.add_text(pos, text, style);
    }

    pub fn finish(mut self) -> SvgCanvas {
        self.canvas.add_watermark(PointD::new(780.0, 780.0));
        self.canvas
    }
}
