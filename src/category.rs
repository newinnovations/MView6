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

use crate::image::colors::Color;

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u32)]
pub enum Category {
    Folder = 0,
    Favorite = 1,
    Image = 2,
    Archive = 3,
    Document = 4,
    Trash = 5,
    Unsupported = 6,
}

impl Category {
    pub fn determine(filename: &str, is_dir: bool) -> Self {
        if is_dir {
            return Self::Folder;
        }

        let filename_lower = filename.to_lowercase();

        let archive = filename_lower.ends_with(".zip")
            | filename_lower.ends_with(".rar")
            | filename_lower.ends_with(".mar");
        // | filename_lower.ends_with(".tar")
        // | filename_lower.contains(".tar.")
        // | filename_lower.ends_with(".tgz");

        if archive {
            return Self::Archive;
        }

        if filename_lower.ends_with(".pdf") || filename_lower.ends_with(".epub") {
            return Self::Document;
        }

        let supported = filename_lower.ends_with(".jpg")
            | filename_lower.ends_with(".jpeg")
            | filename_lower.ends_with(".jfif")
            | filename_lower.ends_with(".gif")
            | filename_lower.ends_with(".svg")
            | filename_lower.ends_with(".svgz")
            | filename_lower.ends_with(".webp")
            | filename_lower.ends_with(".heic")
            | filename_lower.ends_with(".avif")
            // | filename_lower.ends_with(".jxl") // TODO
            | filename_lower.ends_with("-1")
            | filename_lower.ends_with(".pcx")
            | filename_lower.ends_with(".png");

        if supported {
            if filename_lower.contains(".hi.") {
                return Self::Favorite;
            }
            if filename_lower.contains(".lo.") {
                return Self::Trash;
            }
            Self::Image
        } else {
            Self::Unsupported
        }
    }

    pub fn id(&self) -> u32 {
        *self as u32
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

    pub fn icon(&self) -> &str {
        match self {
            Self::Folder => "mv6-folder",
            Self::Favorite => "mv6-favorite",
            Self::Image => "mv6-image",
            Self::Archive => "mv6-box",
            Self::Document => "mv6-doc",
            Self::Trash => "mv6-garbage",
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
            Category::Folder => "folder",
            Category::Favorite => "favorite",
            Category::Image => "image",
            Category::Archive => "archive",
            Category::Document => "document",
            Category::Trash => "trash",
            Category::Unsupported => "not supported",
        }
        .into()
    }

    pub fn is_container(&self) -> bool {
        matches!(
            self,
            Category::Folder | Category::Archive | Category::Document
        )
    }
}

impl From<u32> for Category {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Folder,
            1 => Self::Favorite,
            2 => Self::Image,
            3 => Self::Archive,
            4 => Self::Document,
            5 => Self::Trash,
            _ => Self::Unsupported,
        }
    }
}

impl Default for Category {
    fn default() -> Self {
        Self::Unsupported
    }
}
