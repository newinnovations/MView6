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

pub mod analyze_text;
pub mod content_type;
pub mod loader;
pub mod paginated;
pub mod preview;

use cairo::ImageSurface;
use exif::Exif;
use gdk_pixbuf::Pixbuf;
use resvg::usvg::Tree;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use crate::{
    backends::document::PageMode,
    content::{
        paginated::{PaginatedContent, PaginatedContentData},
        preview::PreviewContent,
    },
    file_view::model::{BackendRef, Reference, Row},
    image::{
        animation::{Animation, AnimationImage},
        provider::gdk::GdkImageLoader,
        view::{data::TransparencyMode, Zoom, ZoomMode},
        DualImage, SingleImage,
    },
    rect::{PointD, RectD, SizeD},
    render_thread::model::RenderCommand,
};

static CONTENT_ID: AtomicU32 = AtomicU32::new(1);

fn get_content_id() -> u32 {
    CONTENT_ID.fetch_add(1, Ordering::SeqCst)
}

// Generic macro for any ContentData variant
//
// content_getter!(animation, animation_mut, Animation, AnimationImage);
macro_rules! content_getter {
    ($method_name:ident, $method_name_mut:ident, $variant:ident, $return_type:ty) => {
        pub fn $method_name(&self) -> Option<&$return_type> {
            match &self.data {
                ContentData::$variant(var) => Some(var),
                _ => None,
            }
        }
        pub fn $method_name_mut(&mut self) -> Option<&mut $return_type> {
            match &mut self.data {
                ContentData::$variant(var) => Some(var),
                _ => None,
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct DocContent {
    pub page_mode: PageMode,
    pub size: SizeD,
    pub reference: Reference,
}

impl DocContent {
    pub fn size(&self) -> SizeD {
        self.size
    }

    pub fn has_alpha(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct SvgContent {
    pub tree: Arc<Tree>,
}

impl SvgContent {
    pub fn size(&self) -> SizeD {
        let size = self.tree.size();
        SizeD::new(size.width().into(), size.height().into())
    }

    pub fn has_alpha(&self) -> bool {
        true
    }
}

#[derive(Default)]
pub enum ContentData {
    #[default]
    None,
    Single(SingleImage),
    Dual(DualImage),
    Animation(AnimationImage),
    Svg(SvgContent),
    Doc(DocContent),
    Paginated(PaginatedContent),
    Preview(PreviewContent),
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
    pub data: ContentData,
    pub exif: Option<Exif>,
    pub zoom_mode: ZoomMode,
    pub transparency_mode: TransparencyMode,
    pub tag: Option<String>,
}

impl Content {
    pub fn new_surface(surface: ImageSurface, exif: Option<Exif>) -> Self {
        Content {
            id: get_content_id(),
            data: ContentData::Single(SingleImage::new(surface)),
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_surface_nozoom(surface: ImageSurface) -> Self {
        Content {
            id: get_content_id(),
            data: ContentData::Single(SingleImage::new(surface)),
            exif: None,
            zoom_mode: ZoomMode::NoZoom,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_pixbuf(pixbuf: Option<Pixbuf>, exif: Option<Exif>) -> Self {
        Content {
            id: get_content_id(),
            data: pixbuf.into(),
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
            data: (pixbuf_left, pixbuf_right).into(),
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
            data: (surface_left, surface_right).into(),
            exif,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_animation(animation: Animation) -> Self {
        Content {
            id: get_content_id(),
            data: ContentData::Animation(AnimationImage::new(animation)),
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::NotSpecified,
            tag: None,
        }
    }

    pub fn new_svg(
        tree: Tree,
        tag: Option<String>,
        zoom_mode: ZoomMode,
        transparency_mode: TransparencyMode,
    ) -> Self {
        Content {
            id: get_content_id(),
            data: ContentData::Svg(SvgContent {
                tree: Arc::new(tree),
            }),
            exif: None,
            zoom_mode,
            transparency_mode,
            tag,
        }
    }

    pub fn new_doc(reference: Reference, page_mode: PageMode, size: SizeD) -> Self {
        Content {
            id: get_content_id(),
            data: ContentData::Doc(DocContent {
                page_mode,
                size,
                reference,
            }),
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::White,
            tag: None,
        }
    }

    pub fn new_paginated(mut content: PaginatedContent) -> Self {
        if !content.is_list() {
            content.prepare(); // list prepare after sort
        }
        Content {
            id: get_content_id(),
            data: ContentData::Paginated(content),
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::Black,
            tag: None,
        }
    }

    pub fn new_list(path: &Path, reference: BackendRef, list: Vec<Row>) -> Self {
        let paginated = PaginatedContent::new_list(path, reference, list);
        Self::new_paginated(paginated)
    }

    pub fn new_preview(path: &Path, reference: BackendRef) -> Self {
        let preview = PreviewContent::new(path, reference);
        Content {
            id: get_content_id(),
            data: ContentData::Preview(preview),
            exif: None,
            zoom_mode: ZoomMode::NotSpecified,
            transparency_mode: TransparencyMode::Black,
            tag: None,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn size(&self) -> SizeD {
        match &self.data {
            ContentData::None => Default::default(),
            ContentData::Svg(svg) => svg.size(),
            ContentData::Doc(doc) => doc.size(),
            ContentData::Single(image) => image.size(),
            ContentData::Dual(image) => image.size(),
            ContentData::Animation(image) => image.size(),
            ContentData::Paginated(image) => image.size(),
            ContentData::Preview(image) => image.size(),
        }
    }

    pub fn has_alpha(&self) -> bool {
        match &self.data {
            ContentData::None => false,
            ContentData::Single(single) => single.has_alpha(),
            ContentData::Dual(dual) => dual.has_alpha(),
            ContentData::Animation(animation) => animation.has_alpha(),
            ContentData::Svg(svg) => svg.has_alpha(),
            ContentData::Doc(doc) => doc.has_alpha(),
            ContentData::Paginated(paginated) => paginated.has_alpha(),
            ContentData::Preview(preview) => preview.has_alpha(),
        }
    }

    pub fn needs_render(&self) -> bool {
        matches!(
            &self.data,
            ContentData::Svg(_)
                | ContentData::Doc(_)
                | ContentData::Paginated(_)
                | ContentData::Preview(_)
        )
    }

    pub fn render(&self, zoom: Zoom, viewport: RectD) -> Option<RenderCommand> {
        match &self.data {
            ContentData::Svg(svg) => Some(RenderCommand::RenderSvg(
                self.id(),
                zoom,
                viewport,
                svg.tree.clone(),
            )),
            ContentData::Paginated(paginated) => paginated
                .rendered
                .as_ref()
                .map(|tree| RenderCommand::RenderSvg(self.id(), zoom, viewport, tree.clone())),
            ContentData::Preview(preview) => preview
                .tree
                .as_ref()
                .map(|tree| RenderCommand::RenderSvg(self.id(), zoom, viewport, tree.clone())),
            ContentData::Doc(doc) => Some(RenderCommand::RenderDoc(
                self.id(),
                zoom,
                viewport,
                doc.clone(),
            )),
            _ => None,
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
        if let ContentData::Single(single) = &self.data {
            single.draw_pixbuf(pixbuf, dest_x, dest_y);
        }
    }

    /// Double click handling depends on content
    ///
    /// List
    ///   Go to directory/zip/rar/mar that corresponds to the content
    ///   Select item that was double clicked or first
    ///
    /// Image for doc preview:
    ///   Go to document first page (or memory)
    ///
    /// Text and Raw: do nothing
    ///
    /// None: do nothing
    ///
    /// Image (Single, Dual, Animation, Svg): do nothing
    ///
    /// Doc: do nothing,
    pub fn double_click(&self, position: PointD) -> Reference {
        if let ContentData::Paginated(paginated) = &self.data {
            paginated.double_click(position)
        } else if let ContentData::Preview(preview) = &self.data {
            preview.double_click(position)
        } else {
            Reference::default()
        }
    }

    pub fn sort(&mut self, sort: &str) -> bool {
        if let ContentData::Paginated(paginated) = &mut self.data {
            if let PaginatedContentData::List(list) = &mut paginated.data {
                list.sort(sort);
                paginated.prepare();
                return true;
            }
        }
        false
    }

    pub fn can_enter(&self) -> bool {
        if matches!(self.data, ContentData::Preview(_)) {
            return true;
        }
        if let ContentData::Paginated(paginated) = &self.data {
            if paginated.is_list() {
                return true;
            }
        }
        false
    }

    content_getter!(animation, animation_mut, Animation, AnimationImage);
}
