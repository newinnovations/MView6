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

use resvg::{tiny_skia, usvg::Tree};

use crate::{
    image::{provider::surface::SurfaceData, view::Zoom},
    rect::RectD,
};

pub fn render_svg(zoom: &Zoom, viewport: &RectD, tree: &Tree) -> Option<SurfaceData> {
    let intersection = zoom.intersection(viewport);
    if intersection.is_empty() {
        println!("No SVG to show");
        return None;
    }

    let width = intersection.width().ceil() as u32;
    let height = intersection.height().ceil() as u32;

    // Create a high-resolution pixmap based on zoom level
    if let Some(mut pixmap) = tiny_skia::Pixmap::new(width, height) {
        let transform = tiny_skia::Transform::from_scale(zoom.scale() as f32, zoom.scale() as f32)
            .post_translate(-intersection.x0 as f32, -intersection.y0 as f32);

        // Render the SVG at high resolution
        resvg::render(tree, transform, &mut pixmap.as_mut());

        // Convert RGBA to BGRA (swap red and blue channels)
        let mut data = pixmap.take();
        for chunk in data.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap R and B channels
        }

        // Create a Cairo surface from the pixmap data
        Some(SurfaceData::new(
            data,
            cairo::Format::ARgb32,
            width as i32,
            height as i32,
            4 * width as i32,
        ))
    } else {
        None
    }
}
