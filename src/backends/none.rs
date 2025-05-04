// MView6 -- Opiniated image browser written in Rust and GTK4
//
// Copyright (c) 2024 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU General Public License as published by the Free Software Foundation, either version 3
// of the License, or (at your option) any later version.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR
// IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND
// FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
// DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR
// BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
// STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use super::{Image, ImageParams};
use gtk4::ListStore;

use crate::file_view::{Columns, Cursor, Sort};

use super::{Backend, Selection};

#[derive(Clone)]
pub struct NoneBackend {}

impl NoneBackend {
    pub fn new() -> Self {
        NoneBackend {}
    }
}

impl Default for NoneBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for NoneBackend {
    fn class_name(&self) -> &str {
        "Invalid"
    }

    fn is_none(&self) -> bool {
        true
    }

    fn path(&self) -> &str {
        "/invalid"
    }

    fn store(&self) -> ListStore {
        Columns::store()
    }

    fn leave(&self) -> (Box<dyn Backend>, Selection) {
        (Box::new(NoneBackend::new()), Selection::None)
    }

    fn image(&self, _: &Cursor, _: &ImageParams) -> Image {
        Image::default()
    }

    fn set_sort(&self, _sort: &Sort) {}

    fn sort(&self) -> Sort {
        Default::default()
    }
}
