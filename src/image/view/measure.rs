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

use std::cell::Cell;

use cairo::Context;

use crate::{image::view::Zoom, rect::PointD};

const DECIMAL_POINT: &str = ",";
const DPI: f64 = 600.0;
const THICKNESS: f64 = 3.0;
const SIZE: f64 = 30.0;
const ARROW_SIZE: f64 = 12.0;
const OFFSET: f64 = 15.0; // Offset from marker points

#[derive(Default)]
pub struct MeasureTool {
    anchor: Cell<Option<PointD>>,
    point: Cell<PointD>,
}

impl MeasureTool {
    pub fn reset(&self) {
        self.anchor.replace(None);
        self.point.replace(Default::default());
    }

    pub fn set_anchor(&self, anchor: PointD) {
        if self.is_active() {
            self.anchor.replace(None);
        } else {
            self.anchor.replace(Some(anchor));
        }
    }

    pub fn set_point(&self, point: PointD) {
        self.point.replace(point);
    }

    pub fn is_active(&self) -> bool {
        self.anchor.get().is_some()
    }

    pub fn draw(&self, context: &Context, zoom: &Zoom) {
        if let Some(anchor) = self.anchor.get() {
            let point = self.point.get();

            let delta = (point - anchor).scale(2.54 / DPI);
            let text = format!(
                " Δx: {:.3} cm\n Δy: {:.3} cm\ndist: {:.3} cm",
                delta.x(),
                delta.y(),
                delta.length()
            );

            let anchor_screen = zoom.image_to_screen(&anchor);
            let point_screen = zoom.image_to_screen(&point);
            draw_marker(context, anchor_screen);
            draw_marker(context, point_screen);
            draw_arrow(context, anchor_screen, point_screen);
            draw_info_box(context, point_screen, &text);
        }
    }

    pub fn clipboard_text(&self) -> Option<String> {
        if let Some(anchor) = self.anchor.get() {
            let point = self.point.get();
            let delta = (point - anchor).scale(2.54 / DPI);
            Some(
                format!("{:.3}\t{:.3}\t{:.3}", delta.x(), delta.y(), delta.length())
                    .replace(".", DECIMAL_POINT),
            )
        } else {
            None
        }
    }
}

// Draw a plus marker with black outline and white fill for visibility
fn draw_marker(cr: &Context, m: PointD) {
    // Draw black outline (thicker)
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(THICKNESS + 2.0);
    cr.move_to(m.x() - SIZE - 1.0, m.y());
    cr.line_to(m.x() + SIZE + 1.0, m.y());
    let _ = cr.stroke();
    cr.move_to(m.x(), m.y() - SIZE - 1.0);
    cr.line_to(m.x(), m.y() + SIZE + 1.0);
    let _ = cr.stroke();

    // Draw white cross on top
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_line_width(THICKNESS);
    cr.move_to(m.x() - SIZE, m.y());
    cr.line_to(m.x() + SIZE, m.y());
    let _ = cr.stroke();
    cr.move_to(m.x(), m.y() - SIZE);
    cr.line_to(m.x(), m.y() + SIZE);
    let _ = cr.stroke();
}

// Draw arrow with outline for visibility
fn draw_arrow(cr: &gtk4::cairo::Context, m1: PointD, m2: PointD) {
    let d = m2 - m1;
    let length = d.length();
    let angle = d.angle();

    // Normalize direction vector
    let n = d.unscale(length);

    // Calculate start and end points with offset
    let start = m1 + n.scale(OFFSET);
    let end = m2 - n.scale(OFFSET);

    // Calculate arrowhead points
    let arrow_angle = std::f64::consts::PI / 6.0; // 30 degrees
    let arrow_x1 = end.x() - (ARROW_SIZE + THICKNESS) * (angle - arrow_angle).cos();
    let arrow_y1 = end.y() - (ARROW_SIZE + THICKNESS) * (angle - arrow_angle).sin();
    let arrow_x2 = end.x() - (ARROW_SIZE + THICKNESS) * (angle + arrow_angle).cos();
    let arrow_y2 = end.y() - (ARROW_SIZE + THICKNESS) * (angle + arrow_angle).sin();

    // Draw black outline
    cr.set_source_rgb(0.0, 0.0, 0.0);
    // cr.set_line_width(4.0);
    cr.set_line_width(THICKNESS + 2.0);
    cr.move_to(start.x() - n.x(), start.y() - n.y());
    cr.line_to(end.x() + n.x(), end.y() + n.y());
    let _ = cr.stroke();
    cr.move_to(end.x(), end.y());
    cr.line_to(arrow_x1, arrow_y1);
    cr.move_to(end.x(), end.y());
    cr.line_to(arrow_x2, arrow_y2);
    let _ = cr.stroke();

    // Draw white arrow on top
    cr.set_source_rgb(1.0, 1.0, 1.0);
    // cr.set_line_width(2.0);
    cr.set_line_width(THICKNESS);
    cr.move_to(start.x(), start.y());
    cr.line_to(end.x(), end.y());
    let _ = cr.stroke();
    cr.move_to(end.x(), end.y());
    cr.line_to(arrow_x1, arrow_y1);
    cr.move_to(end.x(), end.y());
    cr.line_to(arrow_x2, arrow_y2);
    let _ = cr.stroke();
}

// Draw info box with delta and distance
fn draw_info_box(cr: &Context, pos: PointD, text: &str) {
    cr.set_font_size(14.0);

    // Calculate text dimensions
    let lines: Vec<&str> = text.lines().collect();
    let mut max_width = 0.0;
    for line in &lines {
        if let Ok(extents) = cr.text_extents(line) {
            // dbg!(&extents);
            if extents.width() > max_width {
                max_width = extents.x_advance(); // width();
            }
        }
    }

    let line_height = 18.0;
    let padding = 6.0;
    let box_width = max_width + 2.0 * padding;
    let box_height = lines.len() as f64 * line_height + 2.0 * padding;
    let box_x = pos.x() + 10.0;
    let box_y = pos.y() + 10.0;

    // Draw black outline box
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.rectangle(box_x - 2.0, box_y - 2.0, box_width + 4.0, box_height + 4.0);
    let _ = cr.fill();

    // Draw white background box
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    cr.rectangle(box_x, box_y, box_width, box_height);
    let _ = cr.fill();

    // Draw text
    cr.set_source_rgb(0.0, 0.0, 0.0);
    for (i, line) in lines.iter().enumerate() {
        cr.move_to(
            box_x + padding,
            box_y + padding + (i as f64 + 1.0) * line_height - 4.0,
        );
        let _ = cr.show_text(line);
    }
}
