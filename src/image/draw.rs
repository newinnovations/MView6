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

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface, Operator};
use gdk_pixbuf::Pixbuf;
use gtk4::gdk::pixbuf_get_from_surface;
use resvg::usvg::Tree;

use crate::{
    backends::thumbnail::TMessage,
    content::{
        paginated::{FONT_SIZE, FONT_SIZE_TITLE},
        Content,
    },
    error::{MviewError, MviewResult},
    image::{
        svg::text_sheet::{svg_options, svg_text_sheet, TextSheet},
        view::{data::TransparencyMode, ZoomMode},
    },
    mview6_error,
};

use super::colors::{CairoColorExt, Color};

pub fn draw_text(title: &str, msg: &str, colors: (Color, Color, Color)) -> Content {
    match svg_text_sheet(title, msg, colors) {
        Ok(image) => image,
        Err(e) => {
            println!("Failed to draw text: {e:?}");
            Content::default()
        }
    }
}

pub fn draw_error(path: &Path, error: MviewError) -> Content {
    // println!("{error:#?}");
    // let msg = &format!("{error:?}");
    // match svg_text_sheet(
    //     "error",
    //     msg,
    //     (Color::ErrorBack, Color::ErrorTitle, Color::ErrorMsg),
    // ) {
    //     Ok(image) => image,
    //     Err(e) => {
    //         println!("Failed to draw text: {e:?}");
    //         Content::default()
    //     }
    // }
    let mut sheet = TextSheet::new(800, 800, FONT_SIZE);
    sheet.header(path, FONT_SIZE_TITLE, 54);

    sheet.delta_y(2.0);

    sheet.add_line(
        "ERROR",
        sheet
            .base_style()
            .color(Color::ErrorTitle)
            .font_size(FONT_SIZE_TITLE * 3 / 2),
    );

    sheet.delta_y(1.0);

    for line in format!("{error:#?}").lines() {
        sheet.add_line(line, sheet.base_style().color(Color::ErrorMsg));
    }

    let svg_content = sheet.finish().render();
    match Tree::from_str(&svg_content, &svg_options()) {
        Ok(tree) => Content::new_svg(tree, None, ZoomMode::NotSpecified, TransparencyMode::Black),
        Err(e) => {
            eprintln!("Error creating ErrorContent {e:#?}");
            Content::default()
        }
    }
}

pub fn thumbnail_sheet(width: i32, height: i32, margin: i32, text: &str) -> MviewResult<Content> {
    let surface: ImageSurface = ImageSurface::create(Format::ARgb32, width, height)?;
    let context = Context::new(&surface)?;
    context.color(Color::Black);
    context.paint()?;

    let mut logo_width = margin + logo(&context, 0, 0, 30.0, false)? as i32;

    context.select_font_face("Liberation Sans", FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(20.0);
    let caption_width = context.text_extents(text)?.width() as i32;

    if caption_width + logo_width + margin > width {
        logo_width = 0;
    }

    if caption_width < width {
        context.move_to(
            (width - caption_width - logo_width) as f64 / 2.0,
            (height - margin - 3) as f64,
        );
        context.color(Color::White);
        context.show_text(text)?;
    }

    if logo_width != 0 {
        logo(&context, width - margin, height - margin, 30.0, true)?;
    }

    Ok(Content::new_surface_nozoom(surface))
}

fn logo(context: &Context, x_right: i32, y: i32, size: f64, draw: bool) -> MviewResult<f64> {
    context.select_font_face("Liberation Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(size);
    let extends = context.text_extents("MView6")?;
    if draw {
        context.move_to(x_right as f64 - extends.width(), y as f64);
        context.color(Color::Red);
        context.show_text("M")?;
        context.color(Color::White);
        context.show_text("View6")?;
        context.stroke()?;
    }
    Ok(extends.width())
}

pub fn text_thumb(message: TMessage) -> MviewResult<Pixbuf> {
    let (color_back, color_title, color_msg) = message.colors;
    let surface: ImageSurface = ImageSurface::create(Format::ARgb32, 175, 175)?;
    let context = Context::new(&surface)?;

    context.color(color_back);
    context.paint()?;

    // logo(&context, width - offset_x, height - 15, 30.0)?;

    context.select_font_face("Liberation Sans", FontSlant::Normal, FontWeight::Bold);
    context.set_font_size(20.0);
    let extends = context.text_extents(message.title())?;
    context.move_to((175.0 - extends.width()) / 2.0, 60.0);
    context.color(color_title);
    context.show_text(message.title())?;

    context.select_font_face("Liberation Sans", FontSlant::Normal, FontWeight::Normal);
    context.set_font_size(14.0);
    context.color(color_msg);

    let target_width = 160.0;

    let extends = context.text_extents(message.message())?;

    if extends.width() > target_width {
        let msg = message.message().chars().collect::<Vec<char>>();

        let mid = msg.len() / 2;

        let mut chars_lost = false;

        let mut m = mid;
        let mut first;
        let mut first_extends;
        loop {
            let a = &msg[..m];
            first = a.iter().collect::<String>();
            first_extends = context.text_extents(&first)?;
            if first_extends.width() <= target_width || m == 0 {
                break;
            }
            chars_lost = true;
            m -= 1;
        }

        let mut m = mid;
        let mut second;
        let mut second_extends;
        loop {
            let a = &msg[m..];
            second = a.iter().collect::<String>();
            second_extends = context.text_extents(&second)?;
            m += 1;
            if second_extends.width() <= target_width || m == msg.len() {
                break;
            }
            chars_lost = true;
        }

        if chars_lost {
            context.move_to(80.0, 121.0);
            context.show_text("...")?;
            context.move_to((175.0 - first_extends.width()) / 2.0, 110.0);
            context.show_text(&first)?;
            context.move_to((175.0 - second_extends.width()) / 2.0, 140.0);
            context.show_text(&second)?;
        } else {
            context.move_to((175.0 - first_extends.width()) / 2.0, 110.0);
            context.show_text(&first)?;
            context.move_to((175.0 - second_extends.width()) / 2.0, 135.0);
            context.show_text(&second)?;
        }
    } else {
        context.move_to((175.0 - extends.width()) / 2.0, 110.0);
        context.show_text(message.message())?;
    }

    match pixbuf_get_from_surface(&surface, 0, 0, 175, 175) {
        Some(pixbuf) => Ok(pixbuf),
        None => mview6_error!("Failed to get pixbuf from surface").into(),
    }
}

pub fn transparency_background() -> MviewResult<ImageSurface> {
    // #define CHECK_MEDIUM 8
    // #define CHECK_BLACK "#000000"
    // #define CHECK_DARK "#555555"
    // 1=#define CHECK_GRAY "#808080"
    // 2=#define CHECK_LIGHT "#cccccc"
    // #define CHECK_WHITE "#ffffff"
    let check_size = 8;

    let surface = ImageSurface::create(Format::ARgb32, check_size * 2, check_size * 2)?;

    let context = Context::new(&surface)?;

    /* Use source operator to make fully transparent work */
    context.set_operator(Operator::Source);

    let check_size = check_size as f64;

    // context.set_source_rgba(0.5, 0.5, 0.5, 1.0);
    context.color(Color::Gray);
    context.rectangle(0.0, 0.0, check_size, check_size);
    context.rectangle(check_size, check_size, check_size, check_size);
    context.fill()?;

    // context.set_source_rgba(0.8, 0.8, 0.8, 1.0);
    context.color(Color::Silver);
    context.rectangle(0.0, check_size, check_size, check_size);
    context.rectangle(check_size, 0.0, check_size, check_size);
    context.fill()?;

    Ok(surface)
}
