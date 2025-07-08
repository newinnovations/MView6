// MView6 -- Opiniated image and pdf browser written in Rust and GTK4
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

use pdfium_render::prelude::Pdfium;
use std::sync::OnceLock;

pub struct SyncPdfium(Option<Pdfium>);

/// SAFETY: This assumes PDFium will only be used from a single thread.
/// Using this from multiple threads simultaneously will cause undefined behavior.
unsafe impl Send for SyncPdfium {}
unsafe impl Sync for SyncPdfium {}

static PDFIUM: OnceLock<SyncPdfium> = OnceLock::new();

pub fn load_pdfium() -> &'static SyncPdfium {
    PDFIUM.get_or_init(|| {
        let pdfium = match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
            "/usr/lib/mview6",
        ))
        .or_else(|_| Pdfium::bind_to_system_library())
        {
            Ok(bindings) => Some(Pdfium::new(bindings)),
            Err(_) => None,
        };
        SyncPdfium(pdfium)
    })
}

impl SyncPdfium {
    pub fn get(&self) -> &Option<Pdfium> {
        &self.0
    }
}

pub fn pdfium() -> &'static Option<Pdfium> {
    load_pdfium().get()
}
