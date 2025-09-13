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

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use resvg::usvg::Tree;

use crate::{
    content::paginated::{FONT_SIZE, FONT_SIZE_TITLE},
    file_view::model::{BackendRef, ItemRef, Reference},
    image::{
        colors::Color,
        svg::text_sheet::{svg_options, TextSheet},
    },
    rect::{PointD, SizeD},
};

pub struct PreviewContent {
    pub path: PathBuf,
    pub reference: BackendRef,
    pub tree: Option<Arc<Tree>>,
}

impl PreviewContent {
    pub fn size(&self) -> SizeD {
        SizeD::new(800.0, 800.0)
    }

    pub fn has_alpha(&self) -> bool {
        false
    }

    pub fn new(path: &Path, reference: BackendRef) -> Self {
        let mut sheet = TextSheet::new(800, 800, FONT_SIZE);
        sheet.header(path, FONT_SIZE_TITLE, 54);

        sheet
            .canvas()
            .add_message(PointD::new(400.0, 360.0), "PDF/EPUB", Color::Glaucous);

        sheet.show_open_text();

        let svg_content = sheet.finish().render();

        PreviewContent {
            path: path.into(),
            reference,
            tree: Tree::from_str(&svg_content, &svg_options())
                .map(Arc::new)
                .ok(),
        }
    }

    pub fn double_click(&self, _position: PointD) -> Reference {
        Reference {
            backend: self.reference.clone(),
            item: ItemRef::None,
        }
    }
}
