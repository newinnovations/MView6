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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContentType {
    Avif,
    Epub,
    Gif,
    Heic,
    Jpeg,
    Mar,
    Pcx,
    Pdf,
    Png,
    Rar,
    Svg,
    Webp,
    Zip,
    Unknown,
}

impl ContentType {
    pub fn determine(data: &[u8]) -> Self {
        if data.len() < 4 {
            return ContentType::Unknown; // Not enough bytes to identify
        }

        // ZIP: "PK" (50 4B)
        if data.starts_with(&[0x50, 0x4B]) {
            // Check if it's EPUB (ZIP-based format)
            // Look for "mimetype" string within the first 1024 bytes (common in EPUB)
            if let Some(slice) = data.get(0..std::cmp::min(1024, data.len())) {
                if str::from_utf8(slice)
                    .map(|s| s.contains("mimetype"))
                    .unwrap_or(false)
                {
                    return ContentType::Epub;
                }
            }
            return ContentType::Zip;
        }

        // RAR: Starts with "Rar!\x1A\x07\x00" (RAR 1.5-4.x) or "Rar!\x1A\x07\x01\x00" (RAR 5.0+)
        if data.starts_with(b"Rar!\x1A\x07") {
            return ContentType::Rar;
        }

        // PDF: Starts with "%PDF"
        if data.starts_with(b"%PDF") {
            return ContentType::Pdf;
        }

        // GIF: Starts with "GIF87a" or "GIF89a"
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
            return ContentType::Gif;
        }

        // JPEG: Starts with "\xFF\xD8\xFF"
        if data.starts_with(b"\xFF\xD8\xFF") {
            return ContentType::Jpeg;
        }

        // PNG: Starts with "\x89PNG\r\n\x1A\n"
        if data.starts_with(b"\x89PNG\r\n\x1A\n") {
            return ContentType::Png;
        }

        if data.len() >= 12 {
            // WebP: Starts with "RIFF" followed by length and "WEBP" (at offset 8)
            if data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
                return ContentType::Webp;
            }

            // HEIC: Contains "ftyphei[cxms]" within first 12 bytes
            if data[..data.len().min(256)]
                .windows(7)
                .any(|w| w == b"ftyphei")
            {
                return ContentType::Heic;
            }

            // AVIF: Contains "ftypavif" within first 12 bytes
            if data[..data.len().min(256)]
                .windows(8)
                .any(|w| w == b"ftypavif")
            {
                return ContentType::Avif;
            }
        }

        // SVG: Look for "<svg" within the first 100 bytes (SVG is text-based)
        if let Some(slice) = data.get(0..std::cmp::min(100, data.len())) {
            if str::from_utf8(slice)
                .map(|s| s.contains("<svg"))
                .unwrap_or(false)
            {
                return ContentType::Svg;
            }
        }

        ContentType::Unknown
    }

    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "zip" => ContentType::Zip,
            "rar" => ContentType::Rar,
            "mar" => ContentType::Mar,
            "pdf" => ContentType::Pdf,
            "epub" => ContentType::Epub,
            "jpg" => ContentType::Jpeg,
            "jpeg" => ContentType::Jpeg,
            "jfif" => ContentType::Jpeg,
            "gif" => ContentType::Gif,
            "svg" => ContentType::Svg,
            "svgz" => ContentType::Svg,
            "webp" => ContentType::Webp,
            "heic" => ContentType::Heic,
            "avif" => ContentType::Avif,
            "pcx" => ContentType::Pcx,
            "png" => ContentType::Png,
            _ => ContentType::Unknown,
        }
    }

    // pub fn is_image(&self) -> bool {
    //     matches!(
    //         self,
    //         ContentType::Avif
    //             | ContentType::Gif
    //             | ContentType::Heic
    //             | ContentType::Jpeg
    //             | ContentType::Pcx
    //             | ContentType::Png
    //             | ContentType::Svg
    //             | ContentType::Webp
    //     )
    // }

    // pub fn is_container(&self) -> bool {
    //     matches!(self, ContentType::Mar | ContentType::Rar | ContentType::Zip)
    // }

    // pub fn is_document(&self) -> bool {
    //     matches!(self, ContentType::Pdf | ContentType::Epub)
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_format() {
        // Test cases for each format
        let zip_data = vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00];
        assert_eq!(ContentType::determine(&zip_data), ContentType::Zip);

        let rar_data = vec![0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
        assert_eq!(ContentType::determine(&rar_data), ContentType::Rar);

        let pdf_data = vec![0x25, 0x50, 0x44, 0x46, 0x2D, 0x31, 0x2E];
        assert_eq!(ContentType::determine(&pdf_data), ContentType::Pdf);

        let gif_data = vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61];
        assert_eq!(ContentType::determine(&gif_data), ContentType::Gif);

        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(ContentType::determine(&jpeg_data), ContentType::Jpeg);

        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(ContentType::determine(&png_data), ContentType::Png);

        let webp_data = vec![
            0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50,
        ];
        assert_eq!(ContentType::determine(&webp_data), ContentType::Webp);

        let heic_data = vec![
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63,
        ];
        assert_eq!(ContentType::determine(&heic_data), ContentType::Heic);

        let svg_data = vec![0x3C, 0x73, 0x76, 0x67, 0x20];
        assert_eq!(ContentType::determine(&svg_data), ContentType::Svg);

        let avif_data = vec![
            0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66,
        ];
        assert_eq!(ContentType::determine(&avif_data), ContentType::Avif);

        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        assert_eq!(ContentType::determine(&unknown_data), ContentType::Unknown);
    }

    #[test]
    fn test_png_detection() {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(ContentType::determine(&png_header), ContentType::Png);
    }

    #[test]
    fn test_jpeg_detection() {
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(ContentType::determine(&jpeg_header), ContentType::Jpeg);
    }

    #[test]
    fn test_pdf_detection() {
        let pdf_header = b"%PDF-1.4".to_vec();
        assert_eq!(ContentType::determine(&pdf_header), ContentType::Pdf);
    }

    #[test]
    fn test_gif_detection() {
        let gif87_header = b"GIF87a".to_vec();
        let gif89_header = b"GIF89a".to_vec();
        assert_eq!(ContentType::determine(&gif87_header), ContentType::Gif);
        assert_eq!(ContentType::determine(&gif89_header), ContentType::Gif);
    }

    #[test]
    fn test_zip_detection() {
        let zip_header = vec![0x50, 0x4B, 0x03, 0x04];
        assert_eq!(ContentType::determine(&zip_header), ContentType::Zip);
    }

    #[test]
    fn test_rar_detection() {
        let rar_header = b"Rar!\x1a\x07\x00".to_vec();
        assert_eq!(ContentType::determine(&rar_header), ContentType::Rar);
    }

    #[test]
    fn test_webp_detection() {
        let mut webp_header = b"RIFF".to_vec();
        webp_header.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // file size placeholder
        webp_header.extend_from_slice(b"WEBP");
        assert_eq!(ContentType::determine(&webp_header), ContentType::Webp);
    }

    #[test]
    fn test_svg_detection() {
        let svg_content =
            b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\">".to_vec();
        assert_eq!(ContentType::determine(&svg_content), ContentType::Svg);
    }

    #[test]
    fn test_unknown_format() {
        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        assert_eq!(ContentType::determine(&unknown_data), ContentType::Unknown);
    }

    #[test]
    fn test_empty_data() {
        let empty_data = vec![];
        assert_eq!(ContentType::determine(&empty_data), ContentType::Unknown);
    }
}
