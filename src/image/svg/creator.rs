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

use crate::{
    image::colors::Color,
    rect::{PointD, VectorD},
};

/// Text alignment options
#[derive(Debug, Clone)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

impl TextAnchor {
    fn to_svg(&self) -> &'static str {
        match self {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        }
    }
}

/// Font weight options
#[derive(Debug, Clone)]
pub enum FontWeight {
    Normal,
    Bold,
}

impl FontWeight {
    fn to_svg(&self) -> &'static str {
        match self {
            FontWeight::Normal => "normal",
            FontWeight::Bold => "bold",
        }
    }
}

/// Text style configuration
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: u32,
    pub font_weight: FontWeight,
    pub fill: Color,
    pub anchor: TextAnchor,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Arial".to_string(),
            font_size: 16,
            font_weight: FontWeight::Normal,
            fill: Color::Black,
            anchor: TextAnchor::Start,
        }
    }
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn font_family(mut self, family: &str) -> Self {
        self.font_family = family.to_string();
        self
    }

    pub fn font_size(mut self, size: u32) -> Self {
        self.font_size = size;
        self
    }

    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    pub fn anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn delta_x(&self, delta: f64) -> VectorD {
        VectorD::new(delta * self.font_size as f64, 0.0)
    }

    pub fn delta_y(&self, delta: f64) -> VectorD {
        VectorD::new(0.0, delta * self.font_size as f64)
    }
}

/// Line style configuration
#[derive(Debug, Clone)]
pub struct LineStyle {
    pub stroke: Color,
    pub stroke_width: f64,
}

impl Default for LineStyle {
    fn default() -> Self {
        Self {
            stroke: Color::Black,
            stroke_width: 1.0,
        }
    }
}

impl LineStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = color;
        self
    }

    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }
}

/// Rectangle style configuration
#[derive(Debug, Clone)]
pub struct RectStyle {
    pub fill: Color,
    pub stroke: Option<Color>,
    pub stroke_width: f64,
}

impl Default for RectStyle {
    fn default() -> Self {
        Self {
            fill: Color::White,
            stroke: None,
            stroke_width: 1.0,
        }
    }
}

impl RectStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill = color;
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.stroke = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }
}

/// SVG element types
#[derive(Debug)]
enum SvgElement {
    Text {
        position: PointD,
        content: String,
        style: TextStyle,
    },
    Line {
        start: PointD,
        end: PointD,
        style: LineStyle,
    },
    Rectangle {
        position: PointD,
        width: f64,
        height: f64,
        style: RectStyle,
    },
    MultiColorText {
        position: PointD,
        spans: Vec<(String, Color)>,
        style: TextStyle,
    },
}

/// Main SVG Canvas for programmatic SVG creation
pub struct SvgCanvas {
    width: u32,
    height: u32,
    background: Color,
    elements: Vec<SvgElement>,
}

impl SvgCanvas {
    /// Create a new SVG canvas with specified dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            background: Color::White,
            elements: Vec::new(),
        }
    }

    /// Set the background color of the canvas
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Add a text element to the canvas
    pub fn add_text(&mut self, position: PointD, text: &str, style: TextStyle) -> &mut Self {
        self.elements.push(SvgElement::Text {
            position,
            content: escape_xml(text),
            style,
        });
        self
    }

    /// Add a line to the canvas
    pub fn add_line(&mut self, start: PointD, end: PointD, style: LineStyle) -> &mut Self {
        self.elements.push(SvgElement::Line { start, end, style });
        self
    }

    /// Add a rectangle to the canvas
    pub fn add_rectangle(
        &mut self,
        position: PointD,
        width: f64,
        height: f64,
        style: RectStyle,
    ) -> &mut Self {
        self.elements.push(SvgElement::Rectangle {
            position,
            width,
            height,
            style,
        });
        self
    }

    /// Add multi-colored text (like the MView6 watermark)
    pub fn add_multicolor_text(
        &mut self,
        position: PointD,
        spans: Vec<(&str, Color)>,
        style: TextStyle,
    ) -> &mut Self {
        let escaped_spans: Vec<(String, Color)> = spans
            .into_iter()
            .map(|(text, color)| (escape_xml(text), color))
            .collect();

        self.elements.push(SvgElement::MultiColorText {
            position,
            spans: escaped_spans,
            style,
        });
        self
    }

    /// Add a title text with predefined styling
    pub fn add_title(&mut self, position: PointD, text: &str, color: Color) -> &mut Self {
        let style = TextStyle::new()
            .font_family("Ubuntu")
            .font_size(85)
            .font_weight(FontWeight::Bold)
            .fill(color)
            .anchor(TextAnchor::Middle);

        self.add_text(position, text, style);
        self
    }

    /// Add a message text with predefined styling
    pub fn add_message(&mut self, position: PointD, text: &str, color: Color) -> &mut Self {
        let style = TextStyle::new()
            .font_family("Liberation Sans")
            .font_size(70)
            .fill(color)
            .anchor(TextAnchor::Middle);

        self.add_text(position, text, style);
        self
    }

    /// Add the MView6 watermark
    pub fn add_watermark(&mut self, position: PointD) -> &mut Self {
        let style = TextStyle::new()
            .font_family("Liberation Sans")
            .font_size(25)
            .font_weight(FontWeight::Bold)
            .anchor(TextAnchor::End);

        let spans = vec![("M", Color::Red), ("View6", Color::White)];
        self.add_multicolor_text(position, spans, style);
        self
    }

    /// Generate the final SVG string
    pub fn render(&self) -> String {
        let mut svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}"><rect x="0" y="0" width="{}" height="{}" fill="{}"/>"#,
            self.width,
            self.height,
            self.width,
            self.height,
            self.width,
            self.height,
            self.background.to_hex()
        );

        for element in &self.elements {
            match element {
                SvgElement::Text {
                    position,
                    content,
                    style,
                } => {
                    svg.push_str(&format!(
                        r#"<text x="{}" y="{}" text-anchor="{}" font-family="{}" font-size="{}" font-weight="{}" fill="{}">{}</text>"#,
                        position.x(), position.y(), style.anchor.to_svg(), style.font_family,
                        style.font_size, style.font_weight.to_svg(), style.fill.to_hex(), content
                    ));
                }
                SvgElement::Line { start, end, style } => {
                    svg.push_str(&format!(
                        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}" stroke-width="{}"/>"#,
                        start.x(),
                        start.y(),
                        end.x(),
                        end.y(),
                        style.stroke.to_hex(),
                        style.stroke_width
                    ));
                }
                SvgElement::Rectangle {
                    position,
                    width,
                    height,
                    style,
                } => {
                    let stroke_attr = if let Some(stroke_color) = &style.stroke {
                        format!(
                            r#" stroke="{}" stroke-width="{}""#,
                            stroke_color.to_hex(),
                            style.stroke_width
                        )
                    } else {
                        String::new()
                    };

                    svg.push_str(&format!(
                        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"{}/>"#,
                        position.x(),
                        position.y(),
                        width,
                        height,
                        style.fill.to_hex(),
                        stroke_attr
                    ));
                }
                SvgElement::MultiColorText {
                    position,
                    spans,
                    style,
                } => {
                    svg.push_str(&format!(
                        r#"<text x="{}" y="{}" text-anchor="{}" font-family="{}" font-size="{}" font-weight="{}">"#,
                        position.x(),
                        position.y(),
                        style.anchor.to_svg(),
                        style.font_family,
                        style.font_size,
                        style.font_weight.to_svg()
                    ));

                    for (text, color) in spans {
                        svg.push_str(&format!(
                            r#"<tspan fill="{}">{}</tspan>"#,
                            color.to_hex(),
                            text
                        ));
                    }

                    svg.push_str("</text>");
                }
            }
        }

        svg.push_str("</svg>");
        svg
    }

    /// Create a text sheet similar to your original function
    pub fn create_text_sheet(title: &str, message: &str, colors: (Color, Color, Color)) -> String {
        let (bg_color, title_color, msg_color) = colors;

        let mut canvas = SvgCanvas::new(600, 600).background(bg_color);

        canvas
            .add_title(PointD::new(300.0, 100.0), title, title_color)
            .add_message(PointD::new(300.0, 320.0), message, msg_color)
            .add_watermark(PointD::new(580.0, 580.0));

        canvas.render()
    }
}

/// Utility function to escape XML characters
fn escape_xml(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            _ => result.push(c),
        }
    }
    result
}

// Example usage and tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_canvas() {
        let mut canvas = SvgCanvas::new(400, 300).background(Color::Blue);

        canvas.add_text(
            PointD::new(200.0, 50.0),
            "Hello World",
            TextStyle::new()
                .font_size(24)
                .fill(Color::White)
                .anchor(TextAnchor::Middle),
        );

        let svg = canvas.render();
        assert!(svg.contains("Hello World"));
        assert!(svg.contains("width=\"400\""));
    }
}
