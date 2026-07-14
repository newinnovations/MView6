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

use std::{cell::RefCell, path::PathBuf};

use crate::{
    backends::{Backend, ImageParams},
    content::Content,
    file_view::{BackendRef, FileRow, FileStore, ItemRef, Target},
};

struct Parent {
    backend: Box<dyn Backend>,
    target: Target,
}

pub struct NoneBackend {
    store: FileStore,
    parent: RefCell<Option<Parent>>,
    path: RefCell<PathBuf>,
}

impl NoneBackend {
    pub fn new() -> Self {
        NoneBackend {
            store: FileRow::empty_store(),
            parent: RefCell::new(None),
            path: RefCell::new(PathBuf::from("no-file")),
        }
    }

    pub fn set_parent(&self, backend: Box<dyn Backend>, target: Target) {
        let parent = Parent { backend, target };
        *self.parent.borrow_mut() = Some(parent);
    }

    pub fn set_path(&self, path: PathBuf) {
        *self.path.borrow_mut() = path;
    }
}

impl Default for NoneBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for NoneBackend {
    fn class_name(&self) -> &str {
        "NoneBackend"
    }

    fn path(&self) -> PathBuf {
        self.path.borrow().clone()
    }

    fn list(&self) -> FileStore {
        self.store.clone()
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        self.parent
            .borrow_mut()
            .take()
            .map(|parent| (parent.backend, parent.target))
    }

    fn content(&self, _: &ItemRef, _: &ImageParams) -> Content {
        Content::default()
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::None
    }
}
