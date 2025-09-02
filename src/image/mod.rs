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
pub mod svg;
pub mod view;

use cairo::{Context, Filter, Format, ImageSurface, Matrix};
use gdk_pixbuf::Pixbuf;
use gtk4::gdk::prelude::GdkCairoContextExt;
use std::cmp::max;

use crate::{
    image::{animation::AnimationImage, view::Zoom},
    rect::{SizeD, VectorD},
};

#[derive(Debug, Clone)]
pub struct RenderedImage {
    surface: ImageSurface,
    origin: VectorD,
    orig_image_zoom: Zoom,
}

impl RenderedImage {
    pub fn new(surface: ImageSurface, origin: VectorD, orig_image_zoom: Zoom) -> Self {
        Self {
            surface,
            origin,
            orig_image_zoom,
        }
    }

    pub fn draw(&self, context: &Context) {
        let size = self.size();
        context.rectangle(0.0, 0.0, size.width(), size.height());
        let _ = context.set_source_surface(&self.surface, 0.0, 0.0);
        let _ = context.fill();
    }

    pub fn size(&self) -> SizeD {
        SizeD::new(self.surface.width() as f64, self.surface.height() as f64)
    }

    pub fn has_alpha(&self) -> bool {
        self.surface.format() == Format::ARgb32
    }

    /// Creates a Cairo transformation matrix for displaying this rendered image
    ///
    /// It corrects for the situation that the current zoom (scale and position) may have
    /// changed from the original zoom for which this rendering was made. And that until
    /// we have an updated rendering for the current zoom, we must scale and transpose this one.
    pub fn transform_matrix(&self, current_image_zoom: &Zoom) -> Matrix {
        let scale = current_image_zoom.scale() / self.orig_image_zoom.scale();
        let new_origin = current_image_zoom.origin() + self.origin.scale(scale)
            - self.orig_image_zoom.origin().scale(scale);
        let mut zoom = self.orig_image_zoom.clone();
        zoom.set_origin(new_origin);
        zoom.set_zoom_factor(scale);
        zoom.transform_matrix()
    }
}

#[derive(Debug, Clone)]
pub struct SingleImage {
    surface: ImageSurface,
}

impl SingleImage {
    pub fn new(surface: ImageSurface) -> Self {
        Self { surface }
    }

    pub fn surface(self) -> ImageSurface {
        self.surface
    }

    pub fn draw(&self, context: &Context, quality: Filter) {
        let size = self.size();
        context.rectangle(0.0, 0.0, size.width(), size.height());
        let _ = context.set_source_surface(&self.surface, 0.0, 0.0);
        context.source().set_filter(quality);
        let _ = context.fill();
    }

    pub fn size(&self) -> SizeD {
        SizeD::new(self.surface.width() as f64, self.surface.height() as f64)
    }

    pub fn has_alpha(&self) -> bool {
        self.surface.format() == Format::ARgb32
    }

    pub fn transform_matrix(&self, current_image_zoom: &Zoom) -> Matrix {
        current_image_zoom.transform_matrix()
    }

    pub fn draw_pixbuf(&self, pixbuf: &Pixbuf, dest_x: i32, dest_y: i32) {
        if let Ok(ctx) = Context::new(&self.surface) {
            ctx.set_source_pixbuf(pixbuf, dest_x as f64, dest_y as f64);
            let _ = ctx.paint();
        }
    }
}

#[derive(Debug, Clone)]
pub struct DualImage {
    surface_left: ImageSurface,
    surface_right: ImageSurface,
    offset_y_left: f64,
    offset_x_right: f64,
    offset_y_right: f64,
}

impl DualImage {
    pub fn new(surface_left: ImageSurface, surface_right: ImageSurface) -> Self {
        let width_left = surface_left.width() as f64;
        let height_left = surface_left.height() as f64;
        let height_right = surface_right.height() as f64;
        let (offset_y_left, offset_x_right, offset_y_right) = if height_left > height_right {
            (0.0, width_left, (height_left - height_right) / 2.0)
        } else {
            ((height_right - height_left) / 2.0, width_left, 0.0)
        };
        Self {
            surface_left,
            surface_right,
            offset_y_left,
            offset_x_right,
            offset_y_right,
        }
    }

    pub fn draw(&self, context: &Context, quality: Filter) {
        let size = self.size();

        context.rectangle(0.0, 0.0, size.width(), size.height());
        let _ = context.set_source_surface(&self.surface_left, 0.0, self.offset_y_left);
        context.source().set_filter(quality);
        let _ = context.fill();

        context.rectangle(0.0, 0.0, size.width(), size.height());
        let _ = context.set_source_surface(
            &self.surface_right,
            self.offset_x_right,
            self.offset_y_right,
        );
        context.source().set_filter(quality);
        let _ = context.fill();
    }

    pub fn size(&self) -> SizeD {
        SizeD::new(
            (self.surface_left.width() + self.surface_right.width()).into(),
            max(self.surface_left.height(), self.surface_right.height()).into(),
        )
    }

    pub fn has_alpha(&self) -> bool {
        self.surface_left.format() == Format::ARgb32
            || self.surface_right.format() == Format::ARgb32
    }

    pub fn transform_matrix(&self, current_image_zoom: &Zoom) -> Matrix {
        current_image_zoom.transform_matrix()
    }
}

pub enum Image<'a> {
    Single(&'a SingleImage),
    Dual(&'a DualImage),
    Rendered(&'a RenderedImage),
    Animation(&'a AnimationImage),
    None,
}

impl<'a> Image<'a> {
    pub fn draw(&self, context: &Context, quality: Filter) {
        match self {
            Image::Single(image) => image.draw(context, quality),
            Image::Dual(image) => image.draw(context, quality),
            Image::Rendered(image) => image.draw(context),
            Image::Animation(image) => image.draw(context),
            Image::None => (),
        }
    }

    pub fn has_alpha(&self) -> bool {
        match self {
            Image::Single(image) => image.has_alpha(),
            Image::Dual(image) => image.has_alpha(),
            Image::Rendered(image) => image.has_alpha(),
            Image::Animation(image) => image.has_alpha(),
            Image::None => false,
        }
    }

    pub fn transform_matrix(&self, current_image_zoom: &Zoom) -> Matrix {
        match self {
            Image::Single(image) => image.transform_matrix(current_image_zoom),
            Image::Dual(image) => image.transform_matrix(current_image_zoom),
            Image::Rendered(image) => image.transform_matrix(current_image_zoom),
            Image::Animation(image) => image.transform_matrix(current_image_zoom),
            Image::None => Matrix::identity(),
        }
    }
}
