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

use crate::{
    backends::thumbnail::Message,
    file_view::FileView,
    image::ImageView,
    info_view::InfoView,
    render_thread::{RenderCommand, RenderThread, RenderThreadSender},
    window::window_imp::{panel::Panel, toast::ToastOverlay},
};
use async_channel::Sender;
use gio::{SimpleAction, SimpleActionGroup};
use gtk4::{prelude::*, Button, ScrolledWindow};

#[derive(Debug)]
pub struct MViewWidgets {
    pub(super) hbox: gtk4::Box,
    pub(super) toast_overlay: ToastOverlay,
    pub(super) file_widget: ScrolledWindow,
    pub(super) file_view: FileView,
    pub(super) info_widget: ScrolledWindow,
    pub(super) info_view: InfoView,
    pub(super) image_view: ImageView,
    pub tn_sender: Sender<Message>,
    pub(super) _render_thread: RenderThread,
    pub rt_sender: RenderThreadSender,
    pub(super) actions: SimpleActionGroup,
    pub(super) forward_button_top: Button,
    pub(super) panel: Panel,
}

impl MViewWidgets {
    pub fn set_action_string(&self, action_name: &str, state: &str) {
        if let Some(action) = self.actions.lookup_action(action_name) {
            if let Ok(action) = action.downcast::<SimpleAction>() {
                action.set_state(&state.to_variant());
            }
        }
    }

    pub fn set_action_bool(&self, action_name: &str, state: bool) {
        if let Some(action) = self.actions.lookup_action(action_name) {
            if let Ok(action) = action.downcast::<SimpleAction>() {
                action.set_state(&state.to_variant());
            }
        }
    }

    pub fn get_action_bool(&self, action_name: &str) -> bool {
        self.actions
            .lookup_action(action_name)
            .and_then(|a| a.downcast::<SimpleAction>().ok())
            .and_then(|a| a.state())
            .and_then(|v| v.get::<bool>())
            .unwrap_or_default()
    }

    pub fn get_action_i32(&self, action_name: &str) -> i32 {
        self.actions
            .lookup_action(action_name)
            .and_then(|a| a.downcast::<SimpleAction>().ok())
            .and_then(|a| a.state())
            .and_then(|v| v.get::<String>())
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or_default()
    }

    pub fn rb_send(&self, command: RenderCommand) {
        self.rt_sender.send_blocking(command);
    }

    pub fn update_transform_info(&self) {
        let mirrored = self.image_view.is_mirrored();
        let rotation = self.image_view.rotation();
        self.info_view.update_transform(rotation, mirrored);
        self.set_action_string("rotate", &rotation.to_string());
        self.set_action_bool("mirror", mirrored);
    }
}
