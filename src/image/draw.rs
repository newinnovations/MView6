// MView6 -- Opiniated image and pdf browser written in Rust and GTK4
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

use cairo::{Context, Format, ImageSurface};
use gdk_pixbuf::Pixbuf;
use gtk4::gdk::pixbuf_get_from_surface;

use crate::{
    backends::thumbnail::TMessage,
    error::{MviewError, MviewResult},
    image::{view::ZoomMode, Image},
};

use super::colors::{CairoColorExt, Color};

pub fn draw_text(title: &str, msg: &str, colors: (Color, Color, Color)) -> Image {
    match draw_impl(title, msg, colors) {
        Ok(image) => image,
        Err(e) => {
            println!("Failed to draw text: {:?}", e);
            Image::default()
        }
    }
}

pub fn draw_error(error: MviewError) -> Image {
    println!("{:#?}", error);
    let msg = &format!("{:?}", error);
    match draw_impl(
        "error",
        msg,
        (Color::ErrorBack, Color::ErrorTitle, Color::ErrorMsg),
    ) {
        Ok(image) => image,
        Err(e) => {
            println!("Failed to draw text: {:?}", e);
            Image::default()
        }
    }
}

fn draw_impl(title: &str, msg: &str, colors: (Color, Color, Color)) -> MviewResult<Image> {
    let (_color_back, color_title, color_msg) = colors;
    let surface = ImageSurface::create(Format::ARgb32, 600, 600)?;
    let context = Context::new(&surface)?;

    context.color(Color::Black);
    context.paint()?;

    // context.select_font_face("Arial", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    // context.select_font_face("Ubuntu", cairo::FontSlant::Normal, cairo::FontWeight::Normal); //Bold);
    // "Liberation Sans"

    context.select_font_face("Ubuntu", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    context.set_font_size(85.0);
    let extends = context.text_extents(title)?;
    // dbg!(extends);
    context.color(color_title);
    context.move_to((600.0 - extends.width() - extends.x_bearing()) / 2.0, 100.0);
    context.show_text(title)?;

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
    let mut font_size = 70.0;
    let mut width;
    loop {
        context.set_font_size(font_size);
        let extends = context.text_extents(msg)?;
        width = extends.width() + extends.x_bearing();
        if width < 600.0 || font_size < 12.0 {
            break;
        }
        font_size *= 0.9;
    }

    context.color(color_msg);
    context.move_to((600.0 - width) / 2.0, 320.0);
    context.show_text(msg)?;

    logo(&context, 595, 598, 25.0, true)?;

    Ok(Image::new_surface(&surface, ZoomMode::NoZoom))
}

pub fn thumbnail_sheet(width: i32, height: i32, margin: i32, text: &str) -> MviewResult<Image> {
    let surface: ImageSurface = ImageSurface::create(Format::ARgb32, width, height)?;
    let context = Context::new(&surface)?;
    context.color(Color::Black);
    context.paint()?;

    let mut logo_width = margin + logo(&context, 0, 0, 30.0, false)? as i32;

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
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

    Ok(Image::new_surface(&surface, ZoomMode::NoZoom))
}

fn logo(context: &Context, x_right: i32, y: i32, size: f64, draw: bool) -> MviewResult<f64> {
    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Bold,
    );
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

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Bold,
    );
    context.set_font_size(20.0);
    let extends = context.text_extents(message.title())?;
    context.move_to((175.0 - extends.width()) / 2.0, 60.0);
    context.color(color_title);
    context.show_text(message.title())?;

    context.select_font_face(
        "Liberation Sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
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
        None => Err("Failed to get pixbuf from surface".into()),
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
    context.set_operator(cairo::Operator::Source);

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
