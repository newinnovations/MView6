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

pub mod file_formats;

use std::{collections::HashSet, path::Path};

use crate::image::colors::Color;

const ARCHIVE_EXT: &[&str] = &["zip", "rar", "mar"];
const DOC_EXT: &[&str] = &["pdf", "epub"];
// TODO: -1, jxl
const IMAGE_EXT: &[&str] = &[
    "jpg", "jpeg", "jfif", "gif", "svg", "svgz", "webp", "heic", "avif", "pcx", "png",
];
const VIDEO_EXT: &[&str] = &[
    "webm", "mkv", "flv", "vob", "ogv", "ogg", "rrc", "gifv", "mng", "mov", "avi", "qt", "wmv",
    "yuv", "rm", "asf", "amv", "mp4", "m4p", "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv", "m4v",
    "svi", "3gp", "3g2", "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b", "mod",
];

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
        let ext_low = extension.to_lowercase();
        if ARCHIVE_EXT.contains(&ext_low.as_str()) {
            return Self::Archive;
        }
        if DOC_EXT.contains(&ext_low.as_str()) {
            return Self::Document;
        }
        if IMAGE_EXT.contains(&ext_low.as_str()) {
            return Self::Image;
        }
        if VIDEO_EXT.contains(&ext_low.as_str()) {
            return Self::Video;
        }
        Self::Unsupported
    }
}

impl From<&Path> for FileType {
    fn from(path: &Path) -> Self {
        let extension = path.extension().unwrap_or_default();
        Self::from_extension(&extension.to_string_lossy())
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
            Self::Liked => "mv6-liked",
            Self::Disliked => "mv6-disliked",
            _ => "mv6-unknown",
        }
    }

    pub fn from_icon(icon_name: &str) -> Self {
        if icon_name == "mv6-liked" {
            Self::Liked
        } else if icon_name == "mv6-disliked" {
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

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FileClassification {
    pub file_type: FileType,
    pub preference: Preference,
}

impl FileClassification {
    pub fn new(file_type: FileType, preference: Preference) -> Self {
        FileClassification {
            file_type,
            preference,
        }
    }

    pub fn determine(path: &Path, is_dir: bool) -> Self {
        let file_type = if is_dir {
            FileType::Folder
        } else {
            path.into()
        };

        Self {
            file_type,
            preference: path.into(),
        }
    }

    pub fn file_type_id(&self) -> u32 {
        self.file_type.id()
    }

    // https://www.svgrepo.com/svg/347736/file-directory
    // 40% #2ec27e
    //
    // https://www.svgrepo.com/svg/528877/box
    // 70% #62a0ea
    //
    // https://www.svgrepo.com/svg/511024/image-01
    // 70% #f8e45c
    //
    // https://www.svgrepo.com/svg/458675/favorite
    //
    // https://www.svgrepo.com/svg/533010/trash-alt
    // 70% #ffbe6f
    //
    // https://www.svgrepo.com/svg/523073/trash-bin-minimalistic
    // 10% #f66151
    //
    // https://www.svgrepo.com/svg/355272/status-unknown
    // 70% #c0bfbc
    //
    // https://www.svgrepo.com/svg/533035/bookmark

    pub fn file_type_icon(&self) -> &str {
        self.file_type.icon()
    }

    pub fn preference_icon(&self) -> &str {
        self.preference.icon()
    }

    pub fn show_preference_icon(&self) -> bool {
        self.preference.show_icon()
    }

    pub fn colors(&self) -> (Color, Color, Color) {
        self.file_type.colors()
    }

    pub fn name(&self) -> String {
        self.file_type.name()
    }

    pub fn short(&self) -> String {
        self.file_type.short()
    }

    pub fn is_container(&self) -> bool {
        self.file_type.is_container()
    }
}

impl From<FileType> for FileClassification {
    fn from(file_type: FileType) -> Self {
        Self {
            file_type,
            preference: Preference::Normal,
        }
    }
}
