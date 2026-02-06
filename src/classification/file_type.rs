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

use crate::{classification::file_formats::FileFormat, image::Color};

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum FileType {
    Folder = 0,
    Archive = 1,
    Image = 2,
    Video = 3,
    Document = 4,
    #[default]
    Unsupported = 5,
}

impl From<u32> for FileType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Folder,
            1 => Self::Archive,
            2 => Self::Image,
            3 => Self::Video,
            4 => Self::Document,
            _ => Self::Unsupported,
        }
    }
}

impl FileType {
    pub fn id(&self) -> u32 {
        *self as u32
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Folder => "mv6-folder",
            Self::Image => "mv6-image",
            Self::Video => "mv6-video",
            Self::Archive => "mv6-box",
            Self::Document => "mv6-doc",
            Self::Unsupported => "mv6-unknown",
        }
    }

    pub fn colors(&self) -> (Color, Color, Color) {
        match self {
            Self::Folder => (Color::FolderBack, Color::FolderTitle, Color::FolderMsg),
            Self::Archive => (Color::ArchiveBack, Color::ArchiveTitle, Color::ArchiveMsg),
            Self::Unsupported => (
                Color::UnsupportedBack,
                Color::UnsupportedTitle,
                Color::UnsupportedMsg,
            ),
            _ => (Color::Black, Color::Silver, Color::White),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Folder => "folder",
            Self::Image => "image",
            Self::Video => "video",
            Self::Archive => "archive",
            Self::Document => "document",
            Self::Unsupported => "not supported",
        }
        .into()
    }

    pub fn short(&self) -> String {
        match self {
            Self::Folder => "dir",
            Self::Image => "img",
            Self::Video => "vid",
            Self::Archive => "arc",
            Self::Document => "doc",
            Self::Unsupported => "---",
        }
        .into()
    }

    pub fn is_container(&self) -> bool {
        matches!(self, Self::Folder | Self::Archive | Self::Document)
    }

    pub fn all() -> HashSet<Self> {
        HashSet::from([
            Self::Folder,
            Self::Archive,
            Self::Image,
            Self::Video,
            Self::Document,
            Self::Unsupported,
        ])
    }

    pub fn from_extension(extension: &str) -> Self {
        FileFormat::from_extension(extension).into()
    }
}

impl From<&Path> for FileType {
    fn from(path: &Path) -> Self {
        let extension = path.extension().unwrap_or_default();
        Self::from_extension(&extension.to_string_lossy())
    }
}

impl From<FileFormat> for FileType {
    fn from(file_format: FileFormat) -> Self {
        match file_format {
            FileFormat::Folder => Self::Folder,
            FileFormat::Image(_) => Self::Image,
            FileFormat::Archive(_) => Self::Archive,
            FileFormat::Document(_) => Self::Document,
            FileFormat::Video(_) => Self::Video,
            FileFormat::Unknown => Self::Unsupported,
        }
    }
}
