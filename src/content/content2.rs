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

use std::sync::atomic::{AtomicU32, Ordering};

use crate::{content::Content, file_view::model::Reference, image::view::ZoomMode, rect::SizeD};

pub const _MAX_CONTENT_SIZE: u64 = 50 * 1024 * 1024;

pub enum ContentData {
    // Lines(String, Vec<String>),
    // Raw(Vec<u8>),
    Image(Content),
    // Error(MviewError),
    // Reference,
}

pub struct Content2 {
    id: u32,
    pub reference: Reference,
    pub data: ContentData,
}

impl Default for Content2 {
    fn default() -> Self {
        Self {
            id: get_content_id(),
            reference: Reference::default(),
            data: ContentData::Image(Content::default()),
        }
    }
}

impl Content2 {
    pub fn new(reference: Reference, data: ContentData) -> Self {
        Self {
            id: get_content_id(),
            reference,
            data,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    fn image(&self) -> &Content {
        match &self.data {
            ContentData::Image(image) => image,
        }
    }

    pub fn size(&self) -> SizeD {
        self.image().size()
    }

    pub fn zoom_mode(&self) -> ZoomMode {
        self.image().zoom_mode()
    }
}

static CONTENT_ID: AtomicU32 = AtomicU32::new(1);

fn get_content_id() -> u32 {
    CONTENT_ID.fetch_add(1, Ordering::SeqCst)
}
