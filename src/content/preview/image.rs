// MView6 -- High-performance PDF and photo viewer built with Rust and GTK4
//
// Copyright (c) 2024-2026 Martin van der Werff <github (at) newinnovations.nl>
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

use image::{codecs::jpeg::JpegEncoder, ColorType, DynamicImage};
use std::io::{BufRead, ErrorKind, Result, Write};

use crate::error::MviewResult;

const TARGET_WIDTH: u32 = 1024;
const TARGET_HEIGHT: u32 = 1024;
const QUALITY: u8 = 70;

#[derive(Debug, Clone, PartialEq)]
pub enum PreviewCaption {
    Video { video_length: u32, captured_at: u32 },
    Page { num_pages: u32, page: u32 },
    Unknown { r#type: u32, data: Vec<u8> },
}

impl PreviewCaption {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Self::Video {
                video_length,
                captured_at,
            } => {
                bytes.extend_from_slice(&1_u32.to_le_bytes());
                bytes.extend_from_slice(&video_length.to_le_bytes());
                bytes.extend_from_slice(&captured_at.to_le_bytes());
            }
            Self::Page { num_pages, page } => {
                bytes.extend_from_slice(&2_u32.to_le_bytes());
                bytes.extend_from_slice(&num_pages.to_le_bytes());
                bytes.extend_from_slice(&page.to_le_bytes());
            }
            Self::Unknown { r#type, data } => {
                bytes.extend_from_slice(&r#type.to_le_bytes());
                bytes.extend_from_slice(data);
            }
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "PreviewCaption data too short",
            ));
        }
        let r#type = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        match r#type {
            1 => {
                if bytes.len() != 12 {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        "Invalid video caption size",
                    ));
                }
                let video_length = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
                let captured_at = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
                Ok(Self::Video {
                    video_length,
                    captured_at,
                })
            }
            2 => {
                if bytes.len() != 12 {
                    return Err(std::io::Error::new(
                        ErrorKind::InvalidData,
                        "Invalid page caption size",
                    ));
                }
                let num_pages = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
                let page = u32::from_le_bytes(bytes[8..12].try_into().unwrap());
                Ok(Self::Page { num_pages, page })
            }
            other => {
                let data = bytes[4..].to_vec();
                Ok(Self::Unknown {
                    r#type: other,
                    data,
                })
            }
        }
    }

    pub fn from_legacy_string(caption: String) -> Self {
        if let Ok(page) = caption.parse::<u32>() {
            Self::Page { num_pages: 0, page }
        } else {
            let parts: Vec<&str> = caption.split(':').collect();
            if parts.len() == 3 {
                let sec_parts: Vec<&str> = parts[2].split('.').collect();
                if let (Ok(h), Ok(m), Ok(s)) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<u32>(),
                    sec_parts[0].parse::<u32>(),
                ) {
                    let cs = if sec_parts.len() > 1 {
                        sec_parts[1].parse::<u32>().unwrap_or(0)
                    } else {
                        0
                    };
                    let captured_at = (h * 3600 + m * 60 + s) * 100 + cs;
                    return Self::Video {
                        video_length: 0,
                        captured_at,
                    };
                }
            }
            Self::Unknown {
                r#type: 0,
                data: caption.into_bytes(),
            }
        }
    }

    #[allow(dead_code)]
    pub fn r#type(&self) -> u32 {
        match self {
            Self::Video { .. } => 1,
            Self::Page { .. } => 2,
            Self::Unknown { r#type, .. } => *r#type,
        }
    }
}

impl std::fmt::Display for PreviewCaption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Video {
                video_length,
                captured_at,
            } => {
                let total_seconds = captured_at / 100;
                let centiseconds = captured_at % 100;
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;
                if *video_length > 0 {
                    write!(
                        f,
                        "{:02}:{:02}:{:02}.{:02} (len: {:.2}s)",
                        hours,
                        minutes,
                        seconds,
                        centiseconds,
                        *video_length as f64 / 100.0
                    )
                } else {
                    write!(
                        f,
                        "{:02}:{:02}:{:02}.{:02}",
                        hours, minutes, seconds, centiseconds
                    )
                }
            }
            Self::Page { num_pages, page } => {
                if *num_pages > 0 {
                    write!(f, "Page {} of {}", page, num_pages)
                } else {
                    write!(f, "Page {}", page)
                }
            }
            Self::Unknown { r#type, data } => {
                if *r#type == 0 {
                    if let Ok(s) = std::str::from_utf8(data) {
                        return write!(f, "{}", s);
                    }
                }
                write!(f, "Unknown type {} ({} bytes)", r#type, data.len())
            }
        }
    }
}

pub struct PreviewImage {
    width: u32,
    height: u32,
    caption: PreviewCaption,
    image_jpeg: Vec<u8>,
}

impl PreviewImage {
    pub fn new(img: DynamicImage, caption: PreviewCaption) -> MviewResult<Self> {
        let (orig_width, orig_height) = (img.width(), img.height());

        // Calculate scaling factor to fit within target dimensions while preserving aspect ratio
        let width_scale = TARGET_WIDTH as f64 / orig_width as f64;
        let height_scale = TARGET_HEIGHT as f64 / orig_height as f64;
        let scale = width_scale.min(height_scale);
        let width = (orig_width as f64 * scale) as u32;
        let height = (orig_height as f64 * scale) as u32;

        // Resize the image
        let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);

        let mut image_jpeg = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut image_jpeg, QUALITY);
        encoder.encode(
            &resized.to_rgb8().into_raw(),
            width,
            height,
            ColorType::Rgb8.into(),
        )?;

        Ok(Self {
            width,
            height,
            caption,
            image_jpeg,
        })
    }

    // 4 total length
    // 4 width
    // 4 height
    // 4 length caption
    // 4 length jpeg encoding
    // n bytes of caption
    // n bytes of jpeg encoding

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let caption_bytes = self.caption.to_bytes();

        let total_length = 5 * 4 + caption_bytes.len() + self.image_jpeg.len();
        writer.write_all(&(total_length as u32).to_le_bytes())?; // 4

        writer.write_all(&self.width.to_le_bytes())?; // 4
        writer.write_all(&self.height.to_le_bytes())?; // 4

        writer.write_all(&(caption_bytes.len() as u32).to_le_bytes())?; // 4
        writer.write_all(&(self.image_jpeg.len() as u32).to_le_bytes())?; // 4

        writer.write_all(&caption_bytes)?;
        writer.write_all(&self.image_jpeg)?;

        Ok(())
    }

    pub fn read<T: BufRead>(reader: &mut T, version: u32) -> Result<Self> {
        let mut buf = [0u8; 20];
        reader.read_exact(&mut buf)?;

        let total_length = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let width = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let height = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let caption_length = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let jpeg_length = u32::from_le_bytes(buf[16..20].try_into().unwrap());

        // Sanity check on values
        if total_length > 5_000_000
            || jpeg_length > 4_000_000
            || width > 4096
            || height > 4096
            || caption_length > 16_384
        {
            return Err(ErrorKind::FileTooLarge.into());
        }

        if total_length != 5 * 4 + caption_length + jpeg_length {
            return Err(ErrorKind::InvalidData.into());
        }

        let mut caption_bytes = vec![0u8; caption_length as usize];
        reader.read_exact(&mut caption_bytes)?;

        let mut image_jpeg = vec![0u8; jpeg_length as usize];
        reader.read_exact(&mut image_jpeg)?;

        let caption = if version == 1 {
            let legacy_str = String::from_utf8(caption_bytes).unwrap_or_default();
            PreviewCaption::from_legacy_string(legacy_str)
        } else {
            PreviewCaption::from_bytes(&caption_bytes)?
        };

        Ok(Self {
            width,
            height,
            caption,
            image_jpeg,
        })
    }

    #[allow(dead_code)]
    pub fn caption(&self) -> &PreviewCaption {
        &self.caption
    }

    #[allow(dead_code)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn jpeg_data(&self) -> &[u8] {
        &self.image_jpeg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_caption_video() {
        let caption = PreviewCaption::Video {
            video_length: 360000,
            captured_at: 6000,
        };
        assert_eq!(caption.r#type(), 1);
        let bytes = caption.to_bytes();
        let parsed = PreviewCaption::from_bytes(&bytes).unwrap();
        assert_eq!(caption, parsed);
        assert_eq!(caption.to_string(), "00:01:00.00 (len: 3600.00s)");
    }

    #[test]
    fn test_preview_caption_page() {
        let caption = PreviewCaption::Page {
            num_pages: 200,
            page: 50,
        };
        assert_eq!(caption.r#type(), 2);
        let bytes = caption.to_bytes();
        let parsed = PreviewCaption::from_bytes(&bytes).unwrap();
        assert_eq!(caption, parsed);
        assert_eq!(caption.to_string(), "Page 50 of 200");
    }

    #[test]
    fn test_legacy_string_parsing() {
        // Page number legacy parsing
        let caption_page = PreviewCaption::from_legacy_string("50".to_string());
        assert_eq!(
            caption_page,
            PreviewCaption::Page {
                num_pages: 0,
                page: 50,
            }
        );

        // Video caption legacy parsing
        let caption_video = PreviewCaption::from_legacy_string("00:01:00.00".to_string());
        assert_eq!(
            caption_video,
            PreviewCaption::Video {
                video_length: 0,
                captured_at: 6000,
            }
        );

        // Fallback legacy parsing
        let caption_unknown = PreviewCaption::from_legacy_string("hello".to_string());
        assert_eq!(
            caption_unknown,
            PreviewCaption::Unknown {
                r#type: 0,
                data: b"hello".to_vec(),
            }
        );
    }
}
