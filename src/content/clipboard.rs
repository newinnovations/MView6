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

use glib::value::ToValue;
use gtk4::gdk::{Clipboard, ContentProvider};

use crate::content::{Content, ContentData};

impl Content {
    pub fn copy_to_clipboard(&self, clipboard: &Clipboard) {
        let texture = if let ContentData::Single(single) = &self.data {
            single.texture()
        } else {
            println!("Texture not available");
            None
        };

        let mut providers = Vec::new();

        if let Some(texture) = texture {
            let image_provider = ContentProvider::for_value(&texture.to_value());
            providers.push(image_provider);
        }

        if let Some(file_path) = self.path.as_ref() {
            let file = gio::File::for_path(file_path);
            let file_list = gio::ListStore::new::<gio::File>();
            file_list.append(&file);

            let uri_provider = ContentProvider::for_value(&file_list.to_value());
            providers.push(uri_provider);

            let text_provider = ContentProvider::for_value(&file_path.to_value());
            providers.push(text_provider);
        } else {
            println!("No file path available for clipboard copy");
        }

        if !providers.is_empty() {
            let union_provider = ContentProvider::new_union(&providers);
            if let Err(e) = clipboard.set_content(Some(&union_provider)) {
                eprintln!("Failed to set clipboard content: {:?}", e);
            } else {
                println!("Content copied to clipboard successfully");
            }
        }
    }
}
