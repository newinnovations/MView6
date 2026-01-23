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

use glib::{clone, subclass::types::ObjectSubclassExt, ControlFlow};

use crate::{
    file_view::{Direction, Target},
    util::remove_source_id,
};

use super::MViewWindowImp;

impl MViewWindowImp {
    pub fn event_shown(&self) {
        if self.is_slideshow_active() {
            println!("Schedule next");
            self.cancel_next_slide();
            self.schedule_next_slide();
        }
    }

    // pub fn event_next_slide(&self, _width: i32, _height: i32) {
    //     self.cancel_next_slide();
    //     self.schedule_next_slide();
    // }

    fn cancel_next_slide(&self) {
        if let Some(id) = self.next_slide_timeout_id.replace(None) {
            if let Err(e) = remove_source_id(&id) {
                println!("remove_source_id: {e}");
            }
        }
    }

    fn schedule_next_slide(&self) {
        self.next_slide_timeout_id
            .replace(Some(glib::timeout_add_local(
                Duration::from_secs(self.get_slideshow_interval() as u64),
                clone!(
                    #[weak(rename_to = this)]
                    self,
                    #[upgrade_or]
                    ControlFlow::Break,
                    move || {
                        this.next_slide_timeout_id.replace(None);
                        this.slidshow_go_next();
                        ControlFlow::Break
                    }
                ),
            )));
    }

    pub fn is_slideshow_active(&self) -> bool {
        self.widgets().get_action_bool("slideshow.active")
    }

    pub fn set_slideshow_active(&self, active: bool) {
        let w = self.widgets();
        w.set_action_bool("slideshow.active", active);
        w.panel.enable_slideshow_mode(active);
        if active {
            self.slidshow_go_next();
        }
    }

    pub fn set_slideshow_interval(&self, new_interval: i32) {
        self.widgets()
            .set_action_string("slideshow.interval", &new_interval.to_string());
        if self.is_slideshow_active() {
            self.slidshow_go_next();
        }
    }

    pub fn get_slideshow_interval(&self) -> i32 {
        self.widgets().get_action_i32("slideshow.interval")
    }

    pub fn slidshow_go_next(&self) {
        println!("Go next");
        let w = self.widgets();
        let filter = self.current_filter.borrow();
        let moved = w
            .file_view
            .navigate_item(Direction::Down, &filter, self.step_size());
        if !moved {
            w.file_view.goto(&Target::First, &filter, &self.obj());
        }
    }
}
