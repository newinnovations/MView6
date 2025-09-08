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

fn analyze_data(data: &[u8]) -> bool {
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
