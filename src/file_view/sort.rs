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

use std::fmt::Display;

use gtk4::SortType;

use crate::SortOptions;

use super::Column;

#[derive(Clone, Copy, Debug, Default)]
pub enum Sort {
    Sorted((Column, SortType)),
    #[default]
    Unsorted,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str_repr())
    }
}

impl Sort {
    pub fn new(column: Column, order: SortType) -> Self {
        Sort::Sorted((column, order))
    }

    pub fn sort_on_category() -> Self {
        Sort::new(Column::FileType, SortType::Ascending)
    }

    pub fn from_args() -> Self {
        let args = crate::ARGS.get().expect("ARGS not set");
        match args.sort {
            SortOptions::TypeAscending => Sort::new(Column::FileType, SortType::Ascending),
            SortOptions::TypeDescending => Sort::new(Column::FileType, SortType::Descending),
            SortOptions::NameAscending => Sort::new(Column::Name, SortType::Ascending),
            SortOptions::NameDescending => Sort::new(Column::Name, SortType::Descending),
            SortOptions::SizeAscending => Sort::new(Column::Size, SortType::Ascending),
            SortOptions::SizeDescending => Sort::new(Column::Size, SortType::Descending),
            SortOptions::DateAscending => Sort::new(Column::Modified, SortType::Ascending),
            SortOptions::DateDescending => Sort::new(Column::Modified, SortType::Descending),
        }
    }

    pub fn str_repr(&self) -> String {
        match self {
            Sort::Sorted((col, order)) => format!(
                "{}{}",
                *col as u32,
                match order {
                    SortType::Ascending => "a",
                    SortType::Descending => "d",
                    _ => "u",
                }
            ),
            Sort::Unsorted => "u".to_string(),
        }
    }
}
