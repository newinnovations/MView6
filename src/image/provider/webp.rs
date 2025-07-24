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

use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Seek},
};

use cairo::ImageSurface;
use exif::Exif;
use image::{RgbImage, RgbaImage};
use image_webp::WebPDecoder;

use crate::{
    error::MviewResult,
    image::{
        animation::{Animation, WebPAnimation},
        provider::image_rs::RsImageLoader,
        Image,
    },
};

pub struct WebP {}

impl WebP {
    pub fn image_from_file(reader: BufReader<File>, exif: Option<Exif>) -> MviewResult<Image> {
        let mut decoder = WebPDecoder::new(reader)?;
        if decoder.is_animated() {
            Ok(Image::new_animation(Animation::WebPFile(Box::new(
                WebPAnimation::<BufReader<File>>::new(decoder)?,
            ))))
        } else {
            Ok(Image::new_surface(Self::read_image(&mut decoder)?, exif))
        }
    }

    pub fn image_from_memory(reader: Cursor<Vec<u8>>, exif: Option<Exif>) -> MviewResult<Image> {
        let mut decoder = WebPDecoder::new(reader)?;
        if decoder.is_animated() {
            Ok(Image::new_animation(Animation::WebPMemory(Box::new(
                WebPAnimation::<Cursor<Vec<u8>>>::new(decoder)?,
            ))))
        } else {
            Ok(Image::new_surface(Self::read_image(&mut decoder)?, exif))
        }
    }

    pub fn read_image<T: BufRead + Seek>(
        decoder: &mut WebPDecoder<T>,
    ) -> MviewResult<ImageSurface> {
        let (width, height) = decoder.dimensions();
        if decoder.has_alpha() {
            let mut img = RgbaImage::new(width, height);
            decoder.read_image(&mut img)?;
            RsImageLoader::rgba8_image_to_surface(&img)
        } else {
            let mut img = RgbImage::new(width, height);
            decoder.read_image(&mut img)?;
            RsImageLoader::rgb8_image_to_surface(&img)
        }
    }

    pub fn read_frame<T: BufRead + Seek>(
        decoder: &mut WebPDecoder<T>,
    ) -> MviewResult<(ImageSurface, u32)> {
        let (width, height) = decoder.dimensions();
        let (surface, delay) = if decoder.has_alpha() {
            let mut img = RgbaImage::new(width, height);
            let delay = decoder.read_frame(&mut img)?;
            (RsImageLoader::rgba8_image_to_surface(&img)?, delay)
        } else {
            let mut img = RgbImage::new(width, height);
            let delay = decoder.read_frame(&mut img)?;
            (RsImageLoader::rgb8_image_to_surface(&img)?, delay)
        };
        Ok((surface, delay))
    }
}
