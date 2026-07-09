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

use gtk4::{
    gio,
    prelude::{Cast, ListModelExt},
};

use crate::file_view::FileView;

use super::{Direction, FileRow, Filter};

impl FileView {
    pub fn navigate_item(&self, direction: Direction, filter: &Filter, count: u32) {
        if let Some((_, new_idx)) = self.try_navigate(direction, filter, count) {
            self.select_index(new_idx);
        }
    }

    pub fn navigate_item_bool(&self, direction: Direction, filter: &Filter, count: u32) -> bool {
        if let Some((_, new_idx)) = self.try_navigate(direction, filter, count) {
            self.select_index(new_idx);
            return true;
        }
        false
    }

    pub fn try_navigate(
        &self,
        direction: Direction,
        filter: &Filter,
        count: u32,
    ) -> Option<(FileRow, u32)> {
        if count == 0 {
            return None;
        }

        let store = self.store()?;
        let (mut file_row, mut position) = self.selected_store(&store)?;

        let mut cnt = count;
        let n_items = store.n_items();

        while !filter.matches(file_row.classification()) {
            position = match direction {
                Direction::Up => position.checked_sub(1),
                Direction::Down => (position + 1 < n_items).then_some(position + 1),
            }?;
            file_row = get_row_at_pos(&store, position)?;
            if filter.matches(file_row.classification()) {
                cnt = count - 1;
                break;
            }
        }

        if cnt == 0 {
            return Some((file_row, position));
        }

        let mut last_match = (file_row, position);

        loop {
            let next_pos = match direction {
                Direction::Up => position.checked_sub(1),
                Direction::Down => (position + 1 < n_items).then_some(position + 1),
            };
            position = match next_pos {
                Some(p) => p,
                None => {
                    if count != cnt {
                        break;
                    }
                    return None;
                }
            };

            file_row = get_row_at_pos(&store, position)?;

            if filter.matches(file_row.classification()) {
                last_match = (file_row, position);
                cnt -= 1;
            } else {
                continue;
            }

            if cnt == 0 {
                break;
            }
        }
        Some(last_match)
    }
}

fn get_row_at_pos(model: &gio::ListModel, position: u32) -> Option<FileRow> {
    model.item(position)?.downcast::<FileRow>().ok()
}
