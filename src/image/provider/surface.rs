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

use cairo::{Format, ImageSurface};

use crate::error::MviewResult;

pub struct Surface {}

impl Surface {
    pub fn from_rgba8_bytes(width: u32, height: u32, rgba8: &[u8]) -> MviewResult<ImageSurface> {
        let mut surface = ImageSurface::create(Format::ARgb32, width as i32, height as i32)?;
        {
            let img_stride = 4 * width as usize;
            let surface_stride = surface.stride() as usize;
            let mut surface_data = surface.data().unwrap();

            for (src_row, dst_row) in rgba8
                .chunks_exact(img_stride)
                .zip(surface_data.chunks_exact_mut(surface_stride))
            {
                for (src_pixel, dst_pixel) in
                    src_row.chunks_exact(4).zip(dst_row.chunks_exact_mut(4))
                {
                    convert_rgba_pixel(src_pixel, dst_pixel);
                }
            }
        }
        surface.mark_dirty();
        Ok(surface)
    }

    pub fn from_bgra8_bytes(width: u32, height: u32, bgra8: &[u8]) -> MviewResult<ImageSurface> {
        let mut surface = ImageSurface::create(Format::ARgb32, width as i32, height as i32)?;
        {
            let img_stride = 4 * width as usize;
            let surface_stride = surface.stride() as usize;
            let mut surface_data = surface.data().unwrap();

            for (src_row, dst_row) in bgra8
                .chunks_exact(img_stride)
                .zip(surface_data.chunks_exact_mut(surface_stride))
            {
                for (src_pixel, dst_pixel) in
                    src_row.chunks_exact(4).zip(dst_row.chunks_exact_mut(4))
                {
                    convert_bgra_pixel(src_pixel, dst_pixel);
                }
            }
        }
        surface.mark_dirty();
        Ok(surface)
    }

    pub fn from_dual_bgra8_bytes(
        left_width: u32,
        left_height: u32,
        left_bgra8: &[u8],
        right_width: u32,
        right_height: u32,
        right_bgra8: &[u8],
    ) -> MviewResult<ImageSurface> {
        // Ensure both images have the same height
        if left_height != right_height {
            return Err("Left and right images must have the same height".into());
        }

        let height = left_height;
        let total_width = left_width + right_width;

        // Validate input data sizes
        let expected_left_size = (left_width * height * 4) as usize;
        let expected_right_size = (right_width * height * 4) as usize;

        if left_bgra8.len() != expected_left_size {
            return Err(format!(
                "Left image data size mismatch: expected {}, got {}",
                expected_left_size,
                left_bgra8.len()
            )
            .into());
        }

        if right_bgra8.len() != expected_right_size {
            return Err(format!(
                "Right image data size mismatch: expected {}, got {}",
                expected_right_size,
                right_bgra8.len()
            )
            .into());
        }

        let mut surface = ImageSurface::create(Format::ARgb32, total_width as i32, height as i32)?;

        {
            let left_stride = 4 * left_width as usize;
            let right_stride = 4 * right_width as usize;
            let surface_stride = surface.stride() as usize;
            let mut surface_data = surface.data().unwrap();

            for row in 0..height as usize {
                let left_row_start = row * left_stride;
                let right_row_start = row * right_stride;
                let surface_row_start = row * surface_stride;

                let left_row = &left_bgra8[left_row_start..left_row_start + left_stride];
                let right_row = &right_bgra8[right_row_start..right_row_start + right_stride];
                let surface_row =
                    &mut surface_data[surface_row_start..surface_row_start + surface_stride];

                // Process left image pixels
                for (src_pixel, dst_pixel) in left_row
                    .chunks_exact(4)
                    .zip(surface_row.chunks_exact_mut(4))
                {
                    convert_bgra_pixel(src_pixel, dst_pixel);
                }

                // Process right image pixels
                let right_start_offset = (left_width * 4) as usize;
                for (src_pixel, dst_pixel) in right_row
                    .chunks_exact(4)
                    .zip(surface_row[right_start_offset..].chunks_exact_mut(4))
                {
                    convert_bgra_pixel(src_pixel, dst_pixel);
                }
            }
        }

        surface.mark_dirty();
        Ok(surface)
    }
}

#[inline]
pub fn convert_rgba_pixel(src: &[u8], dst: &mut [u8]) {
    if src[3] == 255 {
        dst[0] = src[2]; // B
        dst[1] = src[1]; // G
        dst[2] = src[0]; // R
    } else if src[3] == 0 {
        dst[0] = 0; // B
        dst[1] = 0; // G
        dst[2] = 0; // R
    } else {
        let alpha = src[3] as u16;
        dst[0] = ((src[2] as u16 * alpha) / 255) as u8; // B
        dst[1] = ((src[1] as u16 * alpha) / 255) as u8; // G
        dst[2] = ((src[0] as u16 * alpha) / 255) as u8; // R
    }
    dst[3] = src[3]; // A
}

#[inline]
fn convert_bgra_pixel(src: &[u8], dst: &mut [u8]) {
    if src[3] == 255 {
        dst[0] = src[0]; // B
        dst[1] = src[1]; // G
        dst[2] = src[2]; // R
    } else if src[3] == 0 {
        dst[0] = 0; // B
        dst[1] = 0; // G
        dst[2] = 0; // R
    } else {
        let alpha = src[3] as u16;
        dst[0] = ((src[0] as u16 * alpha) / 255) as u8; // B
        dst[1] = ((src[1] as u16 * alpha) / 255) as u8; // G
        dst[2] = ((src[2] as u16 * alpha) / 255) as u8; // R
    }
    dst[3] = src[3]; // A
}
