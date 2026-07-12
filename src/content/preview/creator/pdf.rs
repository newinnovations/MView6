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

use std::path::Path;

use pdfium::{PdfiumDocument, PdfiumRenderConfig};

use crate::{
    content::{PreviewCaption, PreviewContainer, PreviewImage},
    error::MviewResult,
    profile::performance::Performance,
};

const TOTAL_PAGES: i32 = 16;

pub struct PdfPreview {}

impl PdfPreview {
    pub fn create(path: &Path) -> MviewResult<PreviewContainer> {
        let doc = PdfiumDocument::new_from_path(path, None)?;

        let total_pages = doc.page_count();

        let mut images = Vec::new();
        for page_no in select_pages(total_pages, TOTAL_PAGES) {
            let duration = Performance::start();
            let page = doc.page(page_no)?;

            let config = if page.is_landscape() {
                PdfiumRenderConfig::new().with_width(1024)
            } else {
                PdfiumRenderConfig::new().with_height(1024)
            };
            let bitmap = page.render(&config)?;
            let image = PreviewImage::new(
                bitmap.as_rgb8_image()?,
                PreviewCaption::Page {
                    num_pages: total_pages as u32,
                    page: (page_no + 1) as u32,
                },
            )?;
            images.push(image);
            duration.elapsed(&format!("preview {} of {TOTAL_PAGES}", page_no + 1));
        }

        Ok(PreviewContainer::new(images))
    }
}

/// Returns `n` zero-based page indices evenly distributed across a document
/// with `total_pages` pages, including the first (0) and last (total_pages-1)
/// when n >= 2.
///
/// Rules/edge cases:
/// - If `total_pages == 0` or `n == 0`, returns an empty vector.
/// - If `total_pages == 1`, returns `[0]` (regardless of `n`).
/// - If `n > total_pages`, it clamps to `total_pages`.
/// - If `n == 1`, returns `[0]`.
///
/// The indices are computed as:
///   floor(i * (total_pages - 1) / (n - 1)) for i in 0..n
pub fn select_pages(total_pages: i32, n: i32) -> Vec<i32> {
    if total_pages <= 0 || n <= 0 {
        return Vec::new();
    }
    if total_pages == 1 {
        return vec![0];
    }

    let n = n.min(total_pages);

    if n == 1 {
        return vec![0];
    }

    let last = total_pages - 1;
    let denom = n - 1;

    // This generates n indices: i = 0..n-1
    // Guaranteed strictly increasing since last >= denom.
    (0..n).map(|i| i * last / denom).collect()
}

#[cfg(test)]
mod tests {
    use super::select_pages;

    #[test]
    fn empty_or_zero_cases() {
        assert!(select_pages(0, 0).is_empty());
        assert!(select_pages(0, 5).is_empty());
        assert!(select_pages(10, 0).is_empty());
    }

    #[test]
    fn single_page_doc() {
        assert_eq!(select_pages(1, 1), vec![0]);
        assert_eq!(select_pages(1, 5), vec![0]); // still only page 0
    }

    #[test]
    fn n_eq_1() {
        assert_eq!(select_pages(10, 1), vec![0]); // first page only
    }

    #[test]
    fn typical_distributions() {
        // total=10, n=2 -> first & last
        assert_eq!(select_pages(10, 2), vec![0, 9]);

        // total=10, n=3 -> evenly spread
        assert_eq!(select_pages(10, 3), vec![0, 4, 9]);

        // total=10, n=4
        assert_eq!(select_pages(10, 4), vec![0, 3, 6, 9]);

        // total=10, n=5
        assert_eq!(select_pages(10, 5), vec![0, 2, 4, 6, 9]);

        // total=7, n=7 -> all pages
        assert_eq!(select_pages(7, 7), vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn clamp_when_n_exceeds_total() {
        // n is clamped to total_pages
        assert_eq!(select_pages(3, 5), vec![0, 1, 2]);
        assert_eq!(select_pages(5, 10), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn small_examples() {
        assert_eq!(select_pages(2, 2), vec![0, 1]);
        assert_eq!(select_pages(3, 2), vec![0, 2]);
        assert_eq!(select_pages(3, 3), vec![0, 1, 2]);
        assert_eq!(select_pages(4, 3), vec![0, 1, 3]);
    }
}
