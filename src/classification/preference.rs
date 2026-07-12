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

use std::{collections::HashSet, path::Path};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash, gtk4::glib::Enum)]
#[enum_type(name = "mv6Preference")]
#[repr(u32)]
pub enum Preference {
    #[default]
    Normal = 0,
    Liked = 1,
    Disliked = 2,
}

impl Preference {
    pub fn icon(&self) -> &str {
        match self {
            Self::Liked => "mv6-like",
            Self::Disliked => "mv6-dislike",
            _ => "mv6-unknown",
        }
    }

    pub fn from_icon(icon_name: &str) -> Self {
        if icon_name == "mv6-like" {
            Self::Liked
        } else if icon_name == "mv6-dislike" {
            Self::Disliked
        } else {
            Self::Normal
        }
    }

    pub fn show_icon(&self) -> bool {
        matches!(self, Self::Liked | Self::Disliked)
    }

    pub fn all() -> HashSet<Self> {
        HashSet::from([Self::Normal, Self::Liked, Self::Disliked])
    }

    pub fn normal_liked() -> HashSet<Self> {
        HashSet::from([Self::Normal, Self::Liked])
    }
}

impl From<&Path> for Preference {
    fn from(path: &Path) -> Self {
        let filename = path.file_name().unwrap_or_default();
        let filename = filename.to_string_lossy().to_lowercase();
        if filename.contains(".hi.") {
            Self::Liked
        } else if filename.contains(".lo.") {
            Self::Disliked
        } else {
            Self::Normal
        }
    }
}
