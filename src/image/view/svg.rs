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

use cairo::Context;
use resvg::{tiny_skia, usvg::Tree};

use crate::{image::view::Zoom, rect::RectD};

pub fn render_svg(context: &Context, zoom: &Zoom, viewport: &RectD, tree: &Tree) {
    let intersection = zoom.intersection(viewport);
    if intersection.is_empty() {
        println!("No SVG to show");
        return;
    }

    let pixmap_size = zoom.pixmap_size(&intersection);
    let width = pixmap_size.width().ceil() as u32;
    let height = pixmap_size.height().ceil() as u32;

    // Create a high-resolution pixmap based on zoom level
    if let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) {
        let (off_x, off_y) = zoom.pixmap_offset(&intersection);

        let transform =
            tiny_skia::Transform::from_scale(zoom.zoom_factor() as f32, zoom.zoom_factor() as f32)
                .post_translate(off_x as f32, off_y as f32);

        // Render the SVG at high resolution
        resvg::render(tree, transform, &mut pixmap.as_mut());

        // Convert RGBA to BGRA (swap red and blue channels)
        let mut data = pixmap.take();
        for chunk in data.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap R and B channels
        }

        // Create a Cairo surface from the pixmap data
        let stride = cairo::Format::ARgb32.stride_for_width(width).unwrap();
        let surface = cairo::ImageSurface::create_for_data(
            data,
            cairo::Format::ARgb32,
            width as i32,
            height as i32,
            stride,
        );

        match surface {
            Ok(surface) => {
                let _ = context.set_source_surface(&surface, 0.0, 0.0);
                let _ = context.fill();
            }
            Err(e) => {
                eprintln!("Failed to create Cairo surface for SVG: {}", e);
                draw_svg_fallback(context, viewport.width(), viewport.height());
            }
        }
    } else {
        draw_svg_fallback(context, viewport.width(), viewport.height());
    }
}

fn draw_svg_fallback(context: &cairo::Context, width: f64, height: f64) {
    context.set_source_rgba(0.9, 0.9, 0.9, 1.0);
    context.rectangle(10.0, 10.0, width - 20.0, height - 20.0);
    let _ = context.fill();

    // Draw an "SVG" text indicator
    context.set_source_rgba(0.5, 0.5, 0.5, 1.0);
    context.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    context.set_font_size(24.0);

    let text = "SVG";
    let text_extents = context.text_extents(text).unwrap();
    let x = (width - text_extents.width()) / 2.0;
    let y = (height + text_extents.height()) / 2.0;

    context.move_to(x, y);
    let _ = context.show_text(text);
}
