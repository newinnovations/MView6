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
use std::cell::RefCell;

thread_local! {
    static PDFIUM: RefCell<Option<Pdfium>> = const { RefCell::new(None) };
}

fn initialize_pdfium() -> Option<Pdfium> {
    match Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(
        "/usr/lib/mview6",
    ))
    .or_else(|_| Pdfium::bind_to_system_library())
    {
        Ok(bindings) => {
            eprintln!("PDFium loaded successfully");
            Some(Pdfium::new(bindings))
        }
        Err(e) => {
            eprintln!("Failed to load PDFium: {e:?}");
            None
        }
    }
}

/// Execute a closure with read-only access to the PDFium instance
pub fn with_pdfium<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Pdfium) -> R,
{
    PDFIUM.with(|pdfium| {
        let mut pdfium_ref = pdfium.borrow_mut();

        // Initialize if not already done
        if pdfium_ref.is_none() {
            *pdfium_ref = initialize_pdfium();
        }

        // Execute the closure with the pdfium instance
        pdfium_ref.as_ref().map(f)
    })
}

/// Check if PDFium is available without initializing it
pub fn is_pdfium_available() -> bool {
    PDFIUM.with(|pdfium| {
        let pdfium_ref = pdfium.borrow();
        pdfium_ref.is_some()
    })
}

/// Force initialization of PDFium (useful for early error detection)
pub fn ensure_pdfium_loaded() -> bool {
    PDFIUM.with(|pdfium| {
        let mut pdfium_ref = pdfium.borrow_mut();
        if pdfium_ref.is_none() {
            *pdfium_ref = initialize_pdfium();
        }
        pdfium_ref.is_some()
    })
}

// Usage example
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdfium_usage() {
        let result = with_pdfium(|pdfium| pdfium.bindings().version());

        match result {
            Some(version) => println!("PDFium version: {:?}", version),
            None => println!("PDFium not available"),
        }
    }
}
