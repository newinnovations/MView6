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

use std::{cell::Cell, fmt::Display};

use gtk4::{prelude::TreeSortableExtManual, ListStore, SortColumn, SortType};

use super::model::Columns;

#[derive(Clone, Copy, Debug, Default)]
pub enum Sort {
    Sorted((SortColumn, SortType)),
    #[default]
    Unsorted,
}

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sort::Sorted((c, t)) => write!(f, "{}", Sort::to_str(c, t)),
            Sort::Unsorted => write!(f, "Sort(none)"),
        }
    }
}

impl Sort {
    pub fn new(column: SortColumn, order: SortType) -> Self {
        Sort::Sorted((column, order))
    }

    pub fn sort_on_category() -> Self {
        Sort::new(SortColumn::Index(Columns::Cat as u32), SortType::Ascending)
    }

    pub fn on_sort_column_changed(model: &ListStore, current_sort: &Cell<Sort>) {
        let previous_sort = current_sort.get();
        if let Some((new_column, new_order)) = model.sort_column_id() {
            current_sort.set(Sort::new(new_column, new_order));
            if let Sort::Sorted((previous_column, _)) = previous_sort {
                if !previous_column.eq(&new_column) {
                    if let SortColumn::Index(4) = &new_column {
                        model.set_sort_column_id(
                            SortColumn::Index(Columns::Modified as u32),
                            SortType::Descending,
                        )
                    }
                }
            }
        }
    }

    pub fn to_str(col: &SortColumn, order: &SortType) -> String {
        format!(
            "Sort({}, {})",
            match *col {
                SortColumn::Default => "default".to_string(),
                SortColumn::Index(i) => format!("{}", i),
            },
            match *order {
                SortType::Ascending => "asc",
                _ => "des",
            }
        )
    }
}
