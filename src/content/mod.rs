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

pub mod analyze;
pub mod analyze2;
pub mod content2;

use cairo::ImageSurface;
use exif::Exif;
use gdk_pixbuf::Pixbuf;
use resvg::usvg::Tree;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::{
    backends::document::PageMode,
    file_view::model::Reference,
    image::{
        animation::Animation,
        provider::gdk::GdkImageLoader,
        view::{data::TransparencyMode, ZoomMode},
        DualImage, SingleImage,
    },
    rect::SizeD,
};

static CONTENT_ID: AtomicU32 = AtomicU32::new(1);

fn get_content_id() -> u32 {
    CONTENT_ID.fetch_add(1, Ordering::SeqCst)
}

#[derive(Default)]
pub enum ContentData {
    #[default]
    None,
    Single(SingleImage),
    Dual(DualImage),
    Svg(Box<Tree>),
    Doc(PageMode, SizeD),
}

impl From<Option<Pixbuf>> for ContentData {
    fn from(value: Option<Pixbuf>) -> Self {
        GdkImageLoader::surface_from_pixbuf_option(value.as_ref()).into()
    }
}

impl From<(Option<Pixbuf>, Option<Pixbuf>)> for ContentData {
    fn from(value: (Option<Pixbuf>, Option<Pixbuf>)) -> Self {
        let (p1, p2) = value;
        let s1 = GdkImageLoader::surface_from_pixbuf_option(p1.as_ref());
        let s2 = GdkImageLoader::surface_from_pixbuf_option(p2.as_ref());
        (s1, s2).into()
    }
}

impl From<Option<ImageSurface>> for ContentData {
    fn from(value: Option<ImageSurface>) -> Self {
        match value {
            Some(surface) => ContentData::Single(SingleImage::new(surface)),
            None => ContentData::None,
        }
    }
}

impl From<(Option<ImageSurface>, Option<ImageSurface>)> for ContentData {
    fn from(value: (Option<ImageSurface>, Option<ImageSurface>)) -> Self {
        match value {
            (Some(surface), None) => ContentData::Single(SingleImage::new(surface)),
            (None, Some(surface)) => ContentData::Single(SingleImage::new(surface)),
            (Some(surface1), Some(surface2)) => {
                ContentData::Dual(DualImage::new(surface1, surface2))
            }
            (None, None) => ContentData::None,
        }
    }
}

#[derive(Default)]
pub struct Content {
    id: u32,
    pub reference: Reference,
    pub image_data: ContentData,
    pub animation: Animation,
    pub exif: Option<Exif>,
    pub zoom_mode: ZoomMode,
    pub transparency_mode: TransparencyMode,
    pub tag: Option<String>,
}

impl Content {
    pub fn new_surface(surface: ImageSurface, exif: Option<Exif>) -> Self {
        Content {
            id: get_content_id(),
            reference: Default::default(),
            image_data: ContentData::Single(SingleImage::new(surface)),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_surface_nozoom(surface: ImageSurface) -> Self {
        Content {
            id: get_content_id(),
            reference: Default::default(),
            image_data: ContentData::Single(SingleImage::new(surface)),
            animation: Animation::None,
            exif: None,
            zoom_mode: ZoomMode::NoZoom,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_pixbuf(pixbuf: Option<Pixbuf>, exif: Option<Exif>) -> Self {
        Content {
            id: get_content_id(),
            reference: Default::default(),
            image_data: pixbuf.into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_dual_pixbuf(
        pixbuf_left: Option<Pixbuf>,
        pixbuf_right: Option<Pixbuf>,
        exif: Option<Exif>,
    ) -> Self {
        Content {
            id: get_content_id(),
            reference: Default::default(),
            image_data: (pixbuf_left, pixbuf_right).into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_dual_surface(
        surface_left: Option<ImageSurface>,
        surface_right: Option<ImageSurface>,
        exif: Option<Exif>,
    ) -> Self {
        Content {
            id: get_content_id(),
            reference: Default::default(),
            image_data: (surface_left, surface_right).into(),
            animation: Animation::None,
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
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
        Content {
            id: get_content_id(),
            reference: Default::default(),
            image_data: surface.into(),
            animation,
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_svg(
        svg: Tree,
        tag: Option<String>,
        zoom_mode: ZoomMode,
        transparency_mode: TransparencyMode,
    ) -> Self {
        Content {
            id: get_content_id(),
            reference: Default::default(),
            image_data: ContentData::Svg(Box::new(svg)),
            animation: Animation::None,
            exif: None,
            zoom_mode,
            transparency_mode,
            tag,
        }
    }

    pub fn new_doc(reference: Reference, page_mode: PageMode, size: SizeD) -> Self {
        Content {
            id: get_content_id(),
            reference,
            image_data: ContentData::Doc(page_mode, size),
            animation: Animation::None,
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::White,
            tag: None,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn size(&self) -> SizeD {
        match &self.image_data {
            ContentData::None => Default::default(),
            ContentData::Svg(tree) => {
                let size = tree.size();
                SizeD::new(size.width().into(), size.height().into())
            }
            ContentData::Doc(_, size) => *size,
            ContentData::Single(image) => image.size(),
            ContentData::Dual(image) => image.size(),
        }
    }

    pub fn reference(&self) -> &Reference {
        &self.reference
    }

    pub fn has_alpha(&self) -> bool {
        match &self.image_data {
            ContentData::None => false,
            ContentData::Single(single) => single.has_alpha(),
            ContentData::Dual(dual) => dual.has_alpha(),
            ContentData::Svg(_tree) => true,
            ContentData::Doc(_, _size) => true,
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

    pub fn transparency_mode(&self) -> TransparencyMode {
        self.transparency_mode
    }

    pub fn is_movable(&self) -> bool {
        self.zoom_mode != ZoomMode::NoZoom
    }

    pub fn exif(&self) -> Option<&Exif> {
        self.exif.as_ref()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        if let ContentData::Single(single) = &self.image_data {
            single.draw_pixbuf(pixbuf, dest_x, dest_y);
        }
    }
}
