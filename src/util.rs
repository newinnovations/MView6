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

use std::path::Path;

use glib::{ffi::g_source_remove, result_from_gboolean, BoolError, SourceId};

/// Safer alternative to SourceId::remove()
pub fn remove_source_id(id: &SourceId) -> Result<(), BoolError> {
    unsafe { result_from_gboolean!(g_source_remove(id.as_raw()), "Failed to remove source") }
}

pub fn path_to_filename<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub fn path_to_directory<P: AsRef<Path>>(path: P) -> String {
    match path.as_ref().parent() {
        Some(path) => path.to_string_lossy().to_string(),
        None => Default::default(),
    }
}

pub fn path_to_extension<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase()
}

pub fn ellipsis_middle(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    if max_len < 4 {
        // If max_len is too small for ellipses, just truncate
        return s.chars().take(max_len).collect();
    }

    let available_len = max_len - 3;
    let start_len = available_len.div_ceil(2); // Round up for start
    let end_len = available_len / 2; // Round down for end

    let start: String = s.chars().take(start_len).collect();
    let end: String = s.chars().skip(s.chars().count() - end_len).collect();

    format!("{}...{}", start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_string_middle() {
        assert_eq!(ellipsis_middle("Hello", 0), "");
        assert_eq!(ellipsis_middle("Hello", 1), "H");
        assert_eq!(ellipsis_middle("Hello", 2), "He");
        assert_eq!(ellipsis_middle("Hello", 3), "Hel");
        assert_eq!(ellipsis_middle("Hello", 4), "H...");
        assert_eq!(ellipsis_middle("Hello", 5), "Hello");
        assert_eq!(ellipsis_middle("Hello", 6), "Hello");
        assert_eq!(ellipsis_middle("Hello, World!", 9), "Hel...ld!");
        assert_eq!(ellipsis_middle("Hello, World!", 10), "Hell...ld!");
        assert_eq!(ellipsis_middle("Hello, World!", 11), "Hell...rld!");
        assert_eq!(ellipsis_middle("", 5), "");
    }
}
