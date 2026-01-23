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

use crate::window::imp::MViewWindowImp;

#[derive(Clone)]
pub struct Command {
    pub name: &'static str,
    pub shortcut: Option<&'static str>,
    pub action: fn(&MViewWindowImp),
}

pub const COMMANDS: &[Command] = &[
    Command {
        name: "About MView6",
        shortcut: None,
        action: |w| w.show_about_dialog(),
    },
    Command {
        name: "Help screen 1",
        shortcut: None,
        action: |w| w.show_help_page(1),
    },
    Command {
        name: "Help screen 2",
        shortcut: None,
        action: |w| w.show_help_page(2),
    },
    Command {
        name: "Measurements: move endpoints",
        shortcut: Some("tab"),
        action: |w| w.measure_move_endpoints(),
    },
    Command {
        name: "Measurements: toggle feature",
        shortcut: Some("f2"),
        action: |w| w.measure_toggle(),
    },
    Command {
        name: "Navigation: edit filter",
        shortcut: Some("Shift+F"),
        action: |w| w.filter_dialog(),
    },
    Command {
        name: "Open file",
        shortcut: None,
        action: |w| w.open_file(),
    },
    Command {
        name: "PDF backend: MuPDF",
        shortcut: None,
        action: |w| w.change_pdf_provider("mupdf"),
    },
    Command {
        name: "PDF backend: PDFium",
        shortcut: None,
        action: |w| w.change_pdf_provider("pdfium"),
    },
    Command {
        name: "Page mode: Single",
        shortcut: None,
        action: |w| w.change_page_mode("single"),
    },
    Command {
        name: "Page mode: Dual (1, 2-3, 4-5, ...)",
        shortcut: None,
        action: |w| w.change_page_mode("deo"),
    },
    Command {
        name: "Page mode: Dual (1-2, 3-4, 5-6, ...)",
        shortcut: None,
        action: |w| w.change_page_mode("doe"),
    },
    Command {
        name: "Quit MView6",
        shortcut: Some("q"),
        action: |w| w.quit(),
    },
    Command {
        name: "Rotate 90° Clockwise",
        shortcut: None,
        action: |w| w.rotate_image(270),
    },
    Command {
        name: "Rotate 90° Counterclockwise",
        shortcut: None,
        action: |w| w.rotate_image(90),
    },
    Command {
        name: "Rotate 180°",
        shortcut: None,
        action: |w| w.rotate_image(180),
    },
    Command {
        name: "Slideshow interval: 1 second",
        shortcut: None,
        action: |w| w.set_slideshow_interval(1),
    },
    Command {
        name: "Slideshow interval: 3 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(3),
    },
    Command {
        name: "Slideshow interval: 5 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(5),
    },
    Command {
        name: "Slideshow interval: 10 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(10),
    },
    Command {
        name: "Slideshow interval: 30 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(30),
    },
    Command {
        name: "Slideshow interval: 60 seconds",
        shortcut: None,
        action: |w| w.set_slideshow_interval(60),
    },
    Command {
        name: "Start slideshow",
        shortcut: None,
        action: |w| w.set_slideshow_active(true),
    },
    Command {
        name: "Stop slideshow",
        shortcut: None,
        action: |w| w.set_slideshow_active(false),
    },
    Command {
        name: "Thumbnail size: Extra small (80 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(80),
    },
    Command {
        name: "Thumbnail size: Small (100 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(100),
    },
    Command {
        name: "Thumbnail size: Medium (140 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(140),
    },
    Command {
        name: "Thumbnail size: Large (175 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(175),
    },
    Command {
        name: "Thumbnail size: Extra large (250 px)",
        shortcut: None,
        action: |w| w.set_thumbnail_size(250),
    },
    Command {
        name: "Toggle Files pane",
        shortcut: Some("space"),
        action: |w| w.toggle_pane_files(),
    },
    Command {
        name: "Toggle Information pane",
        shortcut: Some("i"),
        action: |w| w.toggle_pane_info(),
    },
    Command {
        name: "Toggle full screen",
        shortcut: Some("F"),
        action: |w| w.toggle_fullscreen(),
    },
    Command {
        name: "Toggle thumbnail view",
        shortcut: Some("t"),
        action: |w| w.toggle_thumbnail_view(),
    },
    Command {
        name: "Transparency background: Black",
        shortcut: None,
        action: |w| w.change_transparency("black"),
    },
    Command {
        name: "Transparency background: Checkerboard",
        shortcut: None,
        action: |w| w.change_transparency("checkerboard"),
    },
    Command {
        name: "Transparency background: White",
        shortcut: None,
        action: |w| w.change_transparency("white"),
    },
    Command {
        name: "Zoom: Fill window",
        shortcut: None,
        action: |w| w.change_zoom("fill"),
    },
    Command {
        name: "Zoom: Fit window",
        shortcut: None,
        action: |w| w.change_zoom("fit"),
    },
    Command {
        name: "Zoom: Maximum zoom",
        shortcut: None,
        action: |w| w.change_zoom("max"),
    },
    Command {
        name: "Zoom: No scaling",
        shortcut: None,
        action: |w| w.change_zoom("nozoom"),
    },
];
