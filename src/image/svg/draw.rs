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

use resvg::usvg::{fontdb, Options, Tree};
use std::{fs::File, io::Read, path::Path};

use crate::{
    error::MviewResult,
    image::{
        colors::Color,
        svg::{creator::SvgCanvas, hexview::HexdumpViewer},
        view::{data::TransparencyMode, ZoomMode},
        Image,
    },
};

fn svg_options<'a>() -> Options<'a> {
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

pub fn svg_text_sheet(title: &str, msg: &str, colors: (Color, Color, Color)) -> MviewResult<Image> {
    let svg_content = SvgCanvas::create_text_sheet(title, msg, colors);
    let tree = Tree::from_str(&svg_content, &svg_options())?;
    Ok(Image::new_svg(
        tree,
        None,
        ZoomMode::NotSpecified,
        TransparencyMode::Black,
    ))
}

pub fn svg_hexdump(name: &str, path: &Path) -> MviewResult<Image> {
    let file = File::open(path)?;
    let mut buffer = Vec::new();
    file.take(1024).read_to_end(&mut buffer)?;
    let hexview = HexdumpViewer::new(buffer);
    let svg_content = hexview.draw(name).render();
    let tree = Tree::from_str(&svg_content, &svg_options())?;
    Ok(Image::new_svg(
        tree,
        None,
        ZoomMode::NotSpecified,
        TransparencyMode::Black,
    ))
}

// fn draw_impl(title: &str, msg: &str, colors: (Color, Color, Color)) -> MviewResult<Tree> {
//     let (color_back, color_title, color_msg) = colors;
//     let svg_content = format!(
//         r#"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="600" viewBox="0 0 600 600">
//             <rect x="0" y="0" width="600" height="600" fill="{}"/>
//             <text x="300" y="100" text-anchor="middle" font-family="Ubuntu" font-size="85" font-weight="bold" fill="{}">{}</text>
//             <text x="300" y="320" text-anchor="middle" font-family="Liberation Sans" font-size="70" fill="{}">{}</text>
//             <text x="580" y="580" text-anchor="end" font-family="Liberation Sans" font-size="25" font-weight="bold">
//                 <tspan fill="red">M</tspan><tspan fill="white">View6</tspan>
//             </text>
//         </svg>"#,
//         color_back.to_hex(),
//         color_title.to_hex(),
//         escape_xml(title),
//         color_msg.to_hex(),
//         escape_xml(msg)
//     );

//     // Parse the SVG string into a Tree
//     let tree = Tree::from_str(&svg_content, &svg_options())?;

//     Ok(tree)
// }

// // Alternative implementation using manual SVG construction (more complex but gives more control)
// #[allow(dead_code)]
// fn draw_impl_manual(title: &str, msg: &str, colors: (Color, Color, Color)) -> MviewResult<Tree> {
//     let (_color_back, color_title, color_msg) = colors;

//     // Calculate font size for message to fit width
//     let mut font_size = 70.0;
//     let estimated_width = estimate_text_width(msg, font_size);
//     if estimated_width > 580.0 {
//         // Leave some margin
//         font_size = font_size * 580.0 / estimated_width;
//         if font_size < 12.0 {
//             font_size = 12.0;
//         }
//     }

//     // Create SVG content as string with calculated font size
//     let svg_content = format!(
//         r#"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="600" viewBox="0 0 600 600">
//             <rect x="0" y="0" width="600" height="600" fill="black"/>
//             <text x="300" y="100" text-anchor="middle" font-family="Ubuntu" font-size="85" font-weight="bold" fill="{}">{}</text>
//             <text x="300" y="320" text-anchor="middle" font-family="Liberation Sans" font-size="{}" fill="{}">{}</text>
//             <text x="595" y="598" text-anchor="end" font-family="Liberation Sans" font-size="25" font-weight="bold">
//                 <tspan fill="red">M</tspan><tspan fill="white">View6</tspan>
//             </text>
//         </svg>"#,
//         color_title.to_hex(),
//         escape_xml(title),
//         font_size,
//         color_msg.to_hex(),
//         escape_xml(msg)
//     );

//     // Parse the SVG string into a Tree
//     let tree = Tree::from_str(&svg_content, &svg_options())?;

//     Ok(tree)
// }

// fn escape_xml(text: &str) -> String {
//     text.replace('&', "&amp;")
//         .replace('<', "&lt;")
//         .replace('>', "&gt;")
//         .replace('"', "&quot;")
//         .replace('\'', "&#39;")
// }

// fn estimate_text_width(text: &str, font_size: f64) -> f64 {
//     // Fallback estimation based on average character width
//     let mut width = 0.0;
//     for ch in text.chars() {
//         width += match ch {
//             'i' | 'l' | 'I' | '1' | '.' | ',' | ';' | ':' | '!' => font_size * 0.3,
//             'j' | 'f' | 't' => font_size * 0.35,
//             'r' => font_size * 0.4,
//             ' ' => font_size * 0.25,
//             'W' | 'M' => font_size * 0.8,
//             'm' | 'w' => font_size * 0.7,
//             _ => font_size * 0.55, // Average character width
//         }
//     }
//     width
// }

// // Function to save the SVG to a file
// fn _save_svg(tree: &Tree, path: &str) -> MviewResult<()> {
//     let svg_data = tree.to_string(&usvg::WriteOptions::default());
//     std::fs::write(path, svg_data)?;
//     Ok(())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::image::colors::Color;

//     #[test]
//     fn test_draw_svg() {
//         let colors = (Color::Black, Color::Red, Color::White);
//         let result = draw_impl("Test Title", "Test Message", colors);
//         assert!(result.is_ok());
//     }

//     #[test]
//     fn test_xml_escaping() {
//         assert_eq!(escape_xml("Hello & World"), "Hello &amp; World");
//         assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
//     }
// }
