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

#![allow(dead_code)]

#[derive(Debug, PartialEq)]
pub enum FileFormat {
    Zip,
    Rar,
    Pdf,
    Epub,
    Gif,
    Jpeg,
    Png,
    Webp,
    Heic,
    Svg,
    Avif,
}

pub fn detect_file_format(data: &[u8]) -> Option<FileFormat> {
    if data.is_empty() {
        return None;
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(FileFormat::Png);
    }

    // JPEG: FF D8 FF
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(FileFormat::Jpeg);
    }

    // GIF: "GIF87a" or "GIF89a"
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return Some(FileFormat::Gif);
    }

    // PDF: "%PDF"
    if data.starts_with(b"%PDF") {
        return Some(FileFormat::Pdf);
    }

    // ZIP: "PK" (50 4B)
    if data.starts_with(&[0x50, 0x4B]) {
        // Check if it's EPUB (ZIP-based format)
        if is_epub(data) {
            return Some(FileFormat::Epub);
        }
        return Some(FileFormat::Zip);
    }

    // RAR: "Rar!" (52 61 72 21)
    if data.starts_with(b"Rar!") {
        return Some(FileFormat::Rar);
    }

    // WEBP: "RIFF" + "WEBP"
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return Some(FileFormat::Webp);
    }

    // HEIC/HEIF: Check for ftyp box with HEIC brand
    if data.len() >= 12
        && &data[4..8] == b"ftyp"
        && (&data[8..12] == b"heic" || &data[8..12] == b"heix" || &data[8..12] == b"heif")
    {
        return Some(FileFormat::Heic);
    }

    // AVIF: Check for ftyp box with AVIF brand
    if data.len() >= 12 && &data[4..8] == b"ftyp" && &data[8..12] == b"avif" {
        return Some(FileFormat::Avif);
    }

    // SVG: Look for XML declaration and svg element
    if is_svg(data) {
        return Some(FileFormat::Svg);
    }

    None
}

fn is_epub(data: &[u8]) -> bool {
    // EPUB files are ZIP archives with a specific structure
    // Look for "mimetype" file entry and "application/epub+zip" content
    if let Some(mimetype_pos) = find_subsequence(data, b"mimetype") {
        // Check if we can find the EPUB mimetype string nearby
        let search_start = mimetype_pos.saturating_sub(100);
        let search_end = (mimetype_pos + 200).min(data.len());
        let search_slice = &data[search_start..search_end];

        return find_subsequence(search_slice, b"application/epub+zip").is_some();
    }
    false
}

fn is_svg(data: &[u8]) -> bool {
    // Convert to string to check for SVG patterns
    if let Ok(text) = std::str::from_utf8(data) {
        let text_lower = text.to_lowercase();
        // Check for common SVG indicators
        return text_lower.contains("<svg")
            && (text_lower.contains("xmlns=\"http://www.w3.org/2000/svg\"")
                || text_lower.contains("xmlns='http://www.w3.org/2000/svg'")
                || text_lower.starts_with("<?xml"));
    }
    false
}

fn find_subsequence(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len())
        .position(|window| window == pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_png_detection() {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00];
        assert_eq!(detect_file_format(&png_header), Some(FileFormat::Png));
    }

    #[test]
    fn test_jpeg_detection() {
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(detect_file_format(&jpeg_header), Some(FileFormat::Jpeg));
    }

    #[test]
    fn test_pdf_detection() {
        let pdf_header = b"%PDF-1.4".to_vec();
        assert_eq!(detect_file_format(&pdf_header), Some(FileFormat::Pdf));
    }

    #[test]
    fn test_gif_detection() {
        let gif87_header = b"GIF87a".to_vec();
        let gif89_header = b"GIF89a".to_vec();
        assert_eq!(detect_file_format(&gif87_header), Some(FileFormat::Gif));
        assert_eq!(detect_file_format(&gif89_header), Some(FileFormat::Gif));
    }

    #[test]
    fn test_zip_detection() {
        let zip_header = vec![0x50, 0x4B, 0x03, 0x04];
        assert_eq!(detect_file_format(&zip_header), Some(FileFormat::Zip));
    }

    #[test]
    fn test_rar_detection() {
        let rar_header = b"Rar!\x1a\x07\x00".to_vec();
        assert_eq!(detect_file_format(&rar_header), Some(FileFormat::Rar));
    }

    #[test]
    fn test_webp_detection() {
        let mut webp_header = b"RIFF".to_vec();
        webp_header.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // file size placeholder
        webp_header.extend_from_slice(b"WEBP");
        assert_eq!(detect_file_format(&webp_header), Some(FileFormat::Webp));
    }

    #[test]
    fn test_svg_detection() {
        let svg_content =
            b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\">".to_vec();
        assert_eq!(detect_file_format(&svg_content), Some(FileFormat::Svg));
    }

    #[test]
    fn test_unknown_format() {
        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        assert_eq!(detect_file_format(&unknown_data), None);
    }

    #[test]
    fn test_empty_data() {
        let empty_data = vec![];
        assert_eq!(detect_file_format(&empty_data), None);
    }
}
