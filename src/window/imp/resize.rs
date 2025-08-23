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

use std::time::Duration;

use glib::{clone, ControlFlow};

use crate::util::remove_source_id;

use super::MViewWindowImp;

const DELAY_CANVAS_RESIZED: u64 = 100;

impl MViewWindowImp {
    pub fn event_canvas_resized(&self, _width: i32, _height: i32) {
        self.cancel_canvas_resized();
        self.schedule_canvas_resized();
    }

    fn cancel_canvas_resized(&self) {
        if let Some(id) = self.canvas_resized_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(&id) {
                println!("remove_source_id: {e}");
            }
        }
    }

    fn schedule_canvas_resized(&self) {
        self.canvas_resized_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_millis(DELAY_CANVAS_RESIZED),
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.canvas_resized_timeout_id.replace(None);
                        let backend = this.backend.borrow();
                        if backend.is_thumbnail() {
                            drop(backend);
                            this.update_thumbnail_backend();
                        }
                        ControlFlow::Break
                    }
                ),
            )));
    }
}
