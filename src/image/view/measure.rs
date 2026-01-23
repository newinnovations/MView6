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

const _DECIMAL_POINT: &str = ",";
const DPI: f64 = 600.0;
const THICKNESS: f64 = 3.0;
const SIZE: f64 = 30.0;
const ARROW_SIZE: f64 = 12.0;
const OFFSET: f64 = 15.0; // Offset from marker points

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementState {
    #[default]
    Idle,
    ShowNextStart,
    ShowNextFinish,
    SetStart,
    SetFinish,
}

#[derive(Default)]
pub struct MeasureTool {
    state: Cell<MeasurementState>,
    start: Cell<PointD>,
    finish: Cell<PointD>,
}

impl MeasureTool {
    pub fn reset(&self) {
        self.state.replace(MeasurementState::Idle);
        self.start.replace(Default::default());
        self.finish.replace(PointD::new(75.0, 75.0));
    }

    pub fn set_point(&self, point: PointD) {
        match self.state.get() {
            MeasurementState::SetStart => {
                self.start.replace(point);
                self.state.replace(MeasurementState::ShowNextStart);
            }
            MeasurementState::SetFinish => {
                self.finish.replace(point);
                self.state.replace(MeasurementState::ShowNextFinish);
            }
            _ => (),
        }
    }

    pub fn set_state(&self, state: MeasurementState) {
        self.state.replace(state);
    }

    pub fn state(&self) -> MeasurementState {
        self.state.get()
    }

    pub fn is_tracking(&self) -> bool {
        matches!(
            self.state.get(),
            MeasurementState::SetStart | MeasurementState::SetFinish
        )
    }

    pub fn draw(&self, context: &Context, zoom: &Zoom, mouse: &PointD) {
        let state = self.state();
        let start = if state == MeasurementState::SetStart {
            zoom.screen_to_image(mouse)
        } else {
            self.start.get()
        };
        let finish = if state == MeasurementState::SetFinish {
            zoom.screen_to_image(mouse)
        } else {
            self.finish.get()
        };

        let delta = (finish - start).scale(2.54 / DPI);
        let text = format!(
            " Δx: {:.3} cm\n Δy: {:.3} cm\ndist: {:.3} cm",
            delta.x(),
            delta.y(),
            delta.length()
        );

        let start = zoom.image_to_screen(&start);
        let finish = zoom.image_to_screen(&finish);
        draw_marker(context, start, true, state == MeasurementState::SetStart);
        draw_marker(context, finish, false, state == MeasurementState::SetFinish);
        draw_arrow(context, start, finish);
        draw_info_box(context, finish, &text);
    }

    pub fn _clipboard_text(&self) -> String {
        let start = self.start.get();
        let finish = self.finish.get();
        let delta = (finish - start).scale(2.54 / DPI);
        format!("{:.3}\t{:.3}\t{:.3}", delta.x(), delta.y(), delta.length())
            .replace(".", _DECIMAL_POINT)
    }
}

// Draw a plus marker with black outline and white fill for visibility
fn draw_marker(cr: &Context, m: PointD, start: bool, highlight: bool) {
    // Draw black outline (thicker)
    if highlight {
        cr.set_source_rgb(0.4, 0.4, 0.4); // fa8128 orange
    } else {
        cr.set_source_rgb(0.0, 0.0, 0.0);
    }
    cr.set_line_width(THICKNESS + 2.0);
    cr.move_to(m.x() - SIZE - 1.0, m.y());
    cr.line_to(m.x() + SIZE + 1.0, m.y());
    let _ = cr.stroke();
    cr.move_to(m.x(), m.y() - SIZE - 1.0);
    cr.line_to(m.x(), m.y() + SIZE + 1.0);
    let _ = cr.stroke();

    // Draw cross on top
    if start {
        cr.set_source_rgb(0.0, 0.969, 0.465); // 00f877 green
    } else {
        // cr.set_source_rgb(0.172, 0.445, 0.684); // 2c72af blue
        cr.set_source_rgb(0.406, 0.832, 0.949); // 68d5f3 blue
    }
    cr.set_line_width(THICKNESS);
    cr.move_to(m.x() - SIZE, m.y());
    cr.line_to(m.x() + SIZE, m.y());
    let _ = cr.stroke();
    cr.move_to(m.x(), m.y() - SIZE);
    cr.line_to(m.x(), m.y() + SIZE);
    let _ = cr.stroke();

    // if highlight {
    //     // cr.set_source_rgb(0.977, 0.504, 0.156); // fa8128 orange
    //     cr.set_source_rgba(1.0,1.0,1.0,0.5); // fa8128 orange
    //     cr.set_line_width(THICKNESS);
    //     cr.move_to(m.x() - SIZE / 2.0, m.y());
    //     cr.line_to(m.x() + SIZE / 2.0, m.y());
    //     let _ = cr.stroke();
    //     cr.move_to(m.x(), m.y() - SIZE / 2.0);
    //     cr.line_to(m.x(), m.y() + SIZE / 2.0);
    //     let _ = cr.stroke();
    // }
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
