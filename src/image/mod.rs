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

pub mod animation;
pub mod colors;
pub mod draw;
pub mod provider;
pub mod view;

use animation::Animation;
use cairo::{Context, Format, ImageSurface};
use exif::Exif;
use gdk_pixbuf::Pixbuf;
use gtk4::gdk::prelude::GdkCairoContextExt;
use resvg::usvg::Tree;
use std::{
    cmp::max,
    sync::atomic::{AtomicU32, Ordering},
};
use view::ZoomMode;

use crate::{image::provider::gdk::GdkImageLoader, rect::SizeD};

static IMAGE_ID: AtomicU32 = AtomicU32::new(1);

fn get_image_id() -> u32 {
    IMAGE_ID.fetch_add(1, Ordering::SeqCst);
    IMAGE_ID.load(Ordering::SeqCst)
}

#[derive(Default)]
pub enum ImageData {
    #[default]
    None,
    Single(ImageSurface),
    Dual(ImageSurface, ImageSurface),
    Svg(Box<Tree>),
}

impl From<Option<Pixbuf>> for ImageData {
    fn from(value: Option<Pixbuf>) -> Self {
        GdkImageLoader::surface_from_pixbuf_option(value.as_ref()).into()
    }
}

impl From<(Option<Pixbuf>, Option<Pixbuf>)> for ImageData {
    fn from(value: (Option<Pixbuf>, Option<Pixbuf>)) -> Self {
        let (p1, p2) = value;
        let s1 = GdkImageLoader::surface_from_pixbuf_option(p1.as_ref());
        let s2 = GdkImageLoader::surface_from_pixbuf_option(p2.as_ref());
        (s1, s2).into()
    }
}

impl From<Option<ImageSurface>> for ImageData {
    fn from(value: Option<ImageSurface>) -> Self {
        match value {
            Some(surface) => ImageData::Single(surface),
            None => ImageData::None,
        }
    }
}

impl From<(Option<ImageSurface>, Option<ImageSurface>)> for ImageData {
    fn from(value: (Option<ImageSurface>, Option<ImageSurface>)) -> Self {
        match value {
            (Some(surface), None) => ImageData::Single(surface),
            (None, Some(surface)) => ImageData::Single(surface),
            (Some(surface1), Some(surface2)) => ImageData::Dual(surface1, surface2),
            (None, None) => ImageData::None,
        }
    }
}

impl ImageData {
    pub fn offset(&self) -> (f64, f64, f64, f64) {
        match self {
            ImageData::Dual(surface_left, surface_right) => {
                let width_left = surface_left.width() as f64;
                let height_left = surface_left.height() as f64;
                let height_right = surface_right.height() as f64;
                if height_left > height_right {
                    (0.0, 0.0, width_left, (height_left - height_right) / 2.0)
                } else {
                    (0.0, (height_right - height_left) / 2.0, width_left, 0.0)
                }
            }
            _ => (0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn is_svg(&self) -> bool {
        matches!(self, ImageData::Svg(_))
    }
}

#[derive(Default)]
pub struct Image {
    id: u32,
    pub image_data: ImageData,
    animation: Animation,
    pub exif: Option<Exif>,
    zoom_mode: ZoomMode,
    tag: Option<String>,
}

impl Image {
    pub fn new_surface(surface: ImageSurface, exif: Option<Exif>) -> Self {
        Image {
            id: get_image_id(),
            image_data: ImageData::Single(surface),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_surface_nozoom(surface: ImageSurface) -> Self {
        Image {
            id: get_image_id(),
            image_data: ImageData::Single(surface),
            animation: Animation::None,
            exif: None,
            zoom_mode: ZoomMode::NoZoom,
            tag: None,
        }
    }

    pub fn new_pixbuf(pixbuf: Option<Pixbuf>, exif: Option<Exif>) -> Self {
        Image {
            id: get_image_id(),
            image_data: pixbuf.into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_dual_pixbuf(
        pixbuf_left: Option<Pixbuf>,
        pixbuf_right: Option<Pixbuf>,
        exif: Option<Exif>,
    ) -> Self {
        Image {
            id: get_image_id(),
            image_data: (pixbuf_left, pixbuf_right).into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_dual_surface(
        surface_left: Option<ImageSurface>,
        surface_right: Option<ImageSurface>,
        exif: Option<Exif>,
    ) -> Self {
        Image {
            id: get_image_id(),
            image_data: (surface_left, surface_right).into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_animation(animation: Animation) -> Self {
        let surface = match &animation {
            Animation::None => None,
            Animation::Gdk(a) => GdkImageLoader::surface_from_pixbuf(&a.pixbuf()).ok(),
            Animation::WebPFile(a) => a.surface_get(0),
            Animation::WebPMemory(a) => a.surface_get(0),
        };
        Image {
            id: get_image_id(),
            image_data: surface.into(),
            animation,
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_svg(svg: Tree, tag: Option<String>, zoom_mode: ZoomMode) -> Self {
        Image {
            id: get_image_id(),
            image_data: ImageData::Svg(Box::new(svg)),
            animation: Animation::None,
            exif: None,
            zoom_mode,
            tag,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn size(&self) -> SizeD {
        match &self.image_data {
            ImageData::None => Default::default(),
            ImageData::Single(pixbuf) => SizeD::new(pixbuf.width().into(), pixbuf.height().into()),
            ImageData::Dual(pixbuf1, pixbuf2) => SizeD::new(
                (pixbuf1.width() + pixbuf2.width()).into(),
                max(pixbuf1.height(), pixbuf2.height()).into(),
            ),
            ImageData::Svg(tree) => {
                let size = tree.size();
                SizeD::new(size.width().into(), size.height().into())
            }
        }
    }

    pub fn has_alpha(&self) -> bool {
        match &self.image_data {
            ImageData::None => false,
            ImageData::Single(pixbuf) => pixbuf.format() == Format::ARgb32,
            ImageData::Dual(pixbuf1, pixbuf2) => {
                pixbuf1.format() == Format::ARgb32 || pixbuf2.format() == Format::ARgb32
            }
            ImageData::Svg(_tree) => true,
        }
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        match &self.tag {
            Some(t) => t.eq(tag),
            None => false,
        }
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        self.zoom_mode
    }

    pub fn is_movable(&self) -> bool {
        self.zoom_mode != ZoomMode::NoZoom
    }

    pub fn exif(&self) -> Option<&Exif> {
        self.exif.as_ref()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        if let ImageData::Single(my_surface) = &self.image_data {
            if let Ok(ctx) = Context::new(my_surface) {
                ctx.set_source_pixbuf(pixbuf, dest_x as f64, dest_y as f64);
                let _ = ctx.paint();
            }
        }
    }

    // pub fn crop_to_max_size(&mut self) {
    //     if let ImageData::Single(pixbuf) = &self.image_data {
    //         if pixbuf.width() > MAX_IMAGE_SIZE || pixbuf.height() > MAX_IMAGE_SIZE {
    //             let new_width = min(pixbuf.width(), MAX_IMAGE_SIZE);
    //             let new_height = min(pixbuf.height(), MAX_IMAGE_SIZE);
    //             let new_pixpuf = Pixbuf::new(
    //                 pixbuf.colorspace(),
    //                 pixbuf.has_alpha(),
    //                 pixbuf.bits_per_sample(),
    //                 new_width,
    //                 new_height,
    //             );
    //             if let Some(new_pixbuf) = &new_pixpuf {
    //                 pixbuf.copy_area(0, 0, new_width, new_height, new_pixbuf, 0, 0);
    //             }
    //             self.image_data = new_pixpuf.into();
    //             self.animation = Animation::None;
    //         }
    //     }
    // }
}
