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

use std::sync::atomic::{AtomicU8, Ordering};

pub mod mupdf;
pub mod pdfium;

const MIN_DOC_HEIGHT: f32 = 32.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PageMode {
    Single,
    #[default]
    DualEvenOdd, // 1, 2-3, 4-5, ...
    DualOddEven, // 1-2, 3-4, 5-6, ...
}

impl From<&str> for PageMode {
    fn from(value: &str) -> Self {
        match value {
            "deo" => PageMode::DualEvenOdd,
            "doe" => PageMode::DualOddEven,
            _ => PageMode::Single,
        }
    }
}

impl From<PageMode> for &str {
    fn from(value: PageMode) -> Self {
        match value {
            PageMode::Single => "single",
            PageMode::DualEvenOdd => "deo",
            PageMode::DualOddEven => "doe",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Pages {
    Single(i32),
    Dual(i32),
}

//   Single(len=4)  DualOdd(len=6)   DualEven(len=7)
//         0              0                0 1
//         1             1 2               2 3
//         2             3 4               4 5
//         3              5                 6

pub fn pages(index: i32, last_page: i32, mode: &PageMode) -> Pages {
    match mode {
        PageMode::Single => Pages::Single(index),
        PageMode::DualEvenOdd => {
            if index == 0 {
                Pages::Single(index)
            } else {
                let left = (index - 1) & !1 | 1;
                if left == last_page {
                    Pages::Single(left)
                } else {
                    Pages::Dual(left)
                }
            }
        }
        PageMode::DualOddEven => {
            let left = index & !1;
            if left == last_page {
                Pages::Single(left)
            } else {
                Pages::Dual(left)
            }
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PdfEngine {
    #[default]
    MuPdf = 0,
    Pdfium = 1,
}

impl From<&str> for PdfEngine {
    fn from(value: &str) -> Self {
        match value {
            "pdfium" => PdfEngine::Pdfium,
            _ => PdfEngine::MuPdf,
        }
    }
}

impl From<PdfEngine> for &str {
    fn from(value: PdfEngine) -> Self {
        match value {
            PdfEngine::MuPdf => "mupdf",
            PdfEngine::Pdfium => "pdfium",
        }
    }
}

impl From<u8> for PdfEngine {
    fn from(value: u8) -> Self {
        match value {
            1 => PdfEngine::Pdfium,
            _ => PdfEngine::MuPdf,
        }
    }
}

impl From<PdfEngine> for u8 {
    fn from(value: PdfEngine) -> Self {
        value as u8
    }
}

static PDF_ENGINE: AtomicU8 = AtomicU8::new(PdfEngine::MuPdf as u8);

pub fn set_pdf_engine(pdf_engine: PdfEngine) {
    PDF_ENGINE.store(pdf_engine.into(), Ordering::Relaxed);
}

pub fn pdf_engine() -> PdfEngine {
    PDF_ENGINE.load(Ordering::Relaxed).into()
}
