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

use std::path::Path;

use crate::{
    classification::{FileType, Preference},
    image::Color,
};

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
