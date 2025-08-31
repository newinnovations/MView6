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

fn analyze_data(data: &[u8], filename: Option<&str>) -> bool {
    // Check file extension first
    if let Some(name) = filename {
        let text_extensions = [
            ".txt", ".md", ".rs", ".py", ".js", ".html", ".css", ".json", ".xml",
        ];
        if text_extensions.iter().any(|ext| name.ends_with(ext)) {
            return true;
        }

        let binary_extensions = [".exe", ".bin", ".jpg", ".png", ".pdf", ".zip"];
        if binary_extensions.iter().any(|ext| name.ends_with(ext)) {
            return false;
        }
    }

    if data.is_empty() {
        return true;
    }

    // Check for UTF-16 BOM and decode
    if is_utf16(data) {
        return true;
    }

    // Check for UTF-8 BOM
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return std::str::from_utf8(&data[3..]).is_ok();
    }

    // Check UTF-8 validity
    if std::str::from_utf8(data).is_ok() {
        // Additional check: ensure reasonable ratio of printable characters
        return has_reasonable_text_ratio(data);
    }

    false
}

fn is_utf16(data: &[u8]) -> bool {
    // Check for UTF-16 BOM
    if data.len() >= 2 {
        let bom = &data[0..2];
        if bom == [0xFF, 0xFE] || bom == [0xFE, 0xFF] {
            return validate_utf16(data);
        }
    }

    // Try to detect UTF-16 without BOM (heuristic)
    if data.len() >= 4 && data.len().is_multiple_of(2) {
        // Check if it looks like UTF-16LE (many null bytes in odd positions)
        let null_in_odd = data.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();
        let total_pairs = data.len() / 2;

        if null_in_odd as f64 / total_pairs as f64 > 0.5 {
            return validate_utf16_le(data);
        }

        // Check if it looks like UTF-16BE (many null bytes in even positions)
        let null_in_even = data.iter().step_by(2).filter(|&&b| b == 0).count();

        if null_in_even as f64 / total_pairs as f64 > 0.5 {
            return validate_utf16_be(data);
        }
    }

    false
}

fn validate_utf16(data: &[u8]) -> bool {
    if data.len() < 2 {
        return false;
    }

    let is_be = data.starts_with(&[0xFE, 0xFF]);
    let text_data = &data[2..]; // Skip BOM

    if is_be {
        validate_utf16_be(text_data)
    } else {
        validate_utf16_le(text_data)
    }
}

fn validate_utf16_le(data: &[u8]) -> bool {
    if !data.len().is_multiple_of(2) {
        return false;
    }

    // Convert to u16 and validate
    let u16_data: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    // Try to decode as UTF-16
    match String::from_utf16(&u16_data) {
        Ok(text) => has_reasonable_text_content(&text),
        Err(_) => false,
    }
}

fn validate_utf16_be(data: &[u8]) -> bool {
    if !data.len().is_multiple_of(2) {
        return false;
    }

    let u16_data: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect();

    match String::from_utf16(&u16_data) {
        Ok(text) => has_reasonable_text_content(&text),
        Err(_) => false,
    }
}

fn has_reasonable_text_ratio(data: &[u8]) -> bool {
    let printable_count = data
        .iter()
        .filter(|&&byte| (32..=126).contains(&byte) || matches!(byte, b'\n' | b'\r' | b'\t'))
        .count();

    let ratio = printable_count as f64 / data.len() as f64;
    ratio > 0.7
}

fn has_reasonable_text_content(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }

    // Check if it contains reasonable text characters
    let printable_chars = text
        .chars()
        .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
        .count();

    let ratio = printable_chars as f64 / text.chars().count() as f64;
    ratio > 0.7
}

fn display_data(data: &[u8], filename: Option<&str>) {
    if analyze_data(data, filename) {
        display_as_text(data);
    } else {
        display_hex_dump(data);
    }
}

fn display_as_text(data: &[u8]) {
    // Try UTF-16 first
    if is_utf16(data) {
        display_utf16_text(data);
        return;
    }

    // Handle UTF-8 BOM
    let text_data = if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &data[3..]
    } else {
        data
    };

    // Display as UTF-8
    match std::str::from_utf8(text_data) {
        Ok(text) => println!("{}", text),
        Err(_) => {
            let text = String::from_utf8_lossy(text_data);
            println!("{}", text);
        }
    }
}

fn display_utf16_text(data: &[u8]) {
    let (is_be, text_start) = if data.starts_with(&[0xFE, 0xFF]) {
        (true, 2)
    } else if data.starts_with(&[0xFF, 0xFE]) {
        (false, 2)
    } else {
        // Try to guess based on null byte pattern
        let null_in_even = data.iter().step_by(2).filter(|&&b| b == 0).count();
        let null_in_odd = data.iter().skip(1).step_by(2).filter(|&&b| b == 0).count();
        (null_in_even > null_in_odd, 0)
    };

    let text_data = &data[text_start..];

    let u16_data: Vec<u16> = text_data
        .chunks_exact(2)
        .map(|chunk| {
            if is_be {
                u16::from_be_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_le_bytes([chunk[0], chunk[1]])
            }
        })
        .collect();

    match String::from_utf16(&u16_data) {
        Ok(text) => println!("{}", text),
        Err(_) => display_hex_dump(data),
    }
}

fn display_hex_dump(_: &[u8]) {}

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
    Unknown,
}

pub fn detect_file_format(data: &[u8]) -> FileFormat {
    if data.len() < 4 {
        return FileFormat::Unknown; // Not enough bytes to identify
    }

    // ZIP: Starts with "PK\x03\x04" or other PKZIP signatures
    if data.starts_with(b"PK\x03\x04")
        || data.starts_with(b"PK\x05\x06")
        || data.starts_with(b"PK\x07\x08")
    {
        return FileFormat::Zip;
    }

    // RAR: Starts with "Rar!\x1A\x07\x00" (RAR 1.5-4.x) or "Rar!\x1A\x07\x01\x00" (RAR 5.0+)
    if data.starts_with(b"Rar!\x1A\x07\x00") || data.starts_with(b"Rar!\x1A\x07\x01\x00") {
        return FileFormat::Rar;
    }

    // PDF: Starts with "%PDF"
    if data.starts_with(b"%PDF") {
        return FileFormat::Pdf;
    }

    // EPUB: EPUB is a ZIP file with specific structure, check for ZIP signature and optional mimetype
    if data.starts_with(b"PK\x03\x04") {
        // Look for "mimetype" string within the first 100 bytes (common in EPUB)
        if let Some(slice) = data.get(0..std::cmp::min(100, data.len())) {
            if str::from_utf8(slice)
                .map(|s| s.contains("mimetype"))
                .unwrap_or(false)
            {
                return FileFormat::Epub;
            }
        }
        // If no mimetype found, assume ZIP (EPUB detection is less certain without deeper parsing)
        return FileFormat::Zip;
    }

    // GIF: Starts with "GIF87a" or "GIF89a"
    if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") {
        return FileFormat::Gif;
    }

    // JPEG: Starts with "\xFF\xD8\xFF"
    if data.starts_with(b"\xFF\xD8\xFF") {
        return FileFormat::Jpeg;
    }

    // PNG: Starts with "\x89PNG\r\n\x1A\n"
    if data.starts_with(b"\x89PNG\r\n\x1A\n") {
        return FileFormat::Png;
    }

    // WebP: Starts with "RIFF" followed by length and "WEBP" (at offset 8)
    if data.len() >= 12 && data.starts_with(b"RIFF") && &data[8..12] == b"WEBP" {
        return FileFormat::Webp;
    }

    // HEIC: Contains "ftypheic" or "ftypmif1" within first 12 bytes
    if data.len() >= 12
        && (data.windows(8).any(|w| w == b"ftypheic") || data.windows(8).any(|w| w == b"ftypmif1"))
    {
        return FileFormat::Heic;
    }

    // SVG: Look for "<svg" within the first 100 bytes (SVG is text-based)
    if let Some(slice) = data.get(0..std::cmp::min(100, data.len())) {
        if str::from_utf8(slice)
            .map(|s| s.contains("<svg"))
            .unwrap_or(false)
        {
            return FileFormat::Svg;
        }
    }

    // AVIF: Contains "ftypavif" within first 12 bytes
    if data.len() >= 12 && data.windows(8).any(|w| w == b"ftypavif") {
        return FileFormat::Avif;
    }

    FileFormat::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_format() {
        // Test cases for each format
        let zip_data = vec![0x50, 0x4B, 0x03, 0x04, 0x14, 0x00];
        assert_eq!(detect_file_format(&zip_data), FileFormat::Zip);

        let rar_data = vec![0x52, 0x61, 0x72, 0x21, 0x1A, 0x07, 0x00];
        assert_eq!(detect_file_format(&rar_data), FileFormat::Rar);

        let pdf_data = vec![0x25, 0x50, 0x44, 0x46, 0x2D, 0x31, 0x2E];
        assert_eq!(detect_file_format(&pdf_data), FileFormat::Pdf);

        let gif_data = vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61];
        assert_eq!(detect_file_format(&gif_data), FileFormat::Gif);

        let jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_file_format(&jpeg_data), FileFormat::Jpeg);

        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_file_format(&png_data), FileFormat::Png);

        let webp_data = vec![
            0x52, 0x49, 0x46, 0x46, 0x00, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50,
        ];
        assert_eq!(detect_file_format(&webp_data), FileFormat::Webp);

        let heic_data = vec![
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x68, 0x65, 0x69, 0x63,
        ];
        assert_eq!(detect_file_format(&heic_data), FileFormat::Heic);

        let svg_data = vec![0x3C, 0x73, 0x76, 0x67, 0x20];
        assert_eq!(detect_file_format(&svg_data), FileFormat::Svg);

        let avif_data = vec![
            0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, 0x61, 0x76, 0x69, 0x66,
        ];
        assert_eq!(detect_file_format(&avif_data), FileFormat::Avif);

        let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
        assert_eq!(detect_file_format(&unknown_data), FileFormat::Unknown);
    }
}
