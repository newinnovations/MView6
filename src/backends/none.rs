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

use std::path::PathBuf;

use super::{Content, ImageParams};

use crate::file_view::{
    model::{BackendRef, ItemRef, Row},
    Cursor,
};

use super::{Backend, Target};

#[derive(Clone)]
pub struct NoneBackend {
    store: Vec<Row>,
}

impl NoneBackend {
    pub fn new() -> Self {
        NoneBackend {
            store: Default::default(),
        }
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

    fn path(&self) -> PathBuf {
        "invalid".into()
    }

    fn list(&self) -> &Vec<Row> {
        &self.store
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        None
    }

    fn content(&self, _: &ItemRef, _: &ImageParams) -> Content {
        Content::default()
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::None
    }

    fn item_ref(&self, _cursor: &Cursor) -> ItemRef {
        ItemRef::Index(0)
    }
}
