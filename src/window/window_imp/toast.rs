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

use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};

use glib::{clone, ControlFlow, SourceId};
use gtk4::{
    glib, prelude::*, Align, Box as GtkBox, Button, Label, Orientation, Overlay, ProgressBar,
    Revealer, RevealerTransitionType,
};

use crate::util::remove_source_id;

const TOAST_TIMEOUT: u32 = 3; // seconds
const TOAST_PROGRESS_INTERVAL: Duration = Duration::from_millis(50);
const TOAST_TRANSITION: u32 = 200;

type DismissCallback = Box<dyn Fn(&Toast)>;

#[derive(Debug)]
pub struct ToastOverlay {
    overlay: Overlay,
}

impl ToastOverlay {
    pub fn new() -> Self {
        Self {
            overlay: Overlay::new(),
        }
    }

    pub fn set_child(&self, child: &impl IsA<gtk4::Widget>) {
        self.overlay.set_child(Some(child));
    }

    pub fn widget(&self) -> &Overlay {
        &self.overlay
    }

    pub fn add_toast(&self, toast: &Toast) {
        toast.show(&self.overlay);
    }
}

/// Bundles the pieces that only exist while a toast is actually on screen.
/// Created in `show`, torn down as a unit in `dismiss`.
struct ActiveToast {
    overlay: Overlay,
    revealer: Revealer,
    timeout_id: SourceId,
    label: Label,
    progress: ProgressBar,
}

pub struct ToastBuilder {
    title: String,
    button_label: Option<String>,
    action_name: Option<String>,
    callback: Option<DismissCallback>,
}

impl ToastBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            button_label: None,
            action_name: None,
            callback: None,
        }
    }

    pub fn button_label(mut self, label: &str) -> Self {
        self.button_label = Some(label.to_string());
        self
    }

    pub fn action_name(mut self, action_name: &str) -> Self {
        self.action_name = Some(action_name.to_string());
        self
    }

    pub fn on_dismissed<F: Fn(&Toast) + 'static>(mut self, callback: F) -> Self {
        self.callback = Some(Box::new(callback));
        self
    }

    pub fn build(self) -> Toast {
        Toast {
            inner: Rc::new(ToastInner {
                title: RefCell::new(self.title),
                button_label: self.button_label,
                action_name: self.action_name,
                callback: self.callback,
                active: RefCell::new(None),
            }),
        }
    }
}

#[derive(Clone)]
pub struct Toast {
    inner: Rc<ToastInner>,
}

impl std::fmt::Debug for Toast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Toast").finish_non_exhaustive()
    }
}

struct ToastInner {
    title: RefCell<String>,
    button_label: Option<String>,
    action_name: Option<String>,
    callback: Option<DismissCallback>,
    active: RefCell<Option<ActiveToast>>,
}

impl Toast {
    pub fn dismiss(&self) {
        let Some(active) = self.inner.active.borrow_mut().take() else {
            // Already dismissed (or never shown).
            return;
        };

        let _ = remove_source_id(&active.timeout_id);

        active.revealer.set_reveal_child(false);
        active.overlay.remove_overlay(&active.revealer);

        if let Some(callback) = self.inner.callback.as_ref() {
            callback(self);
        }
    }

    /// Updates the toast's title text and restarts its countdown timer, without
    /// tearing down the widget or invoking the dismissed callback. Used when a
    /// bulk action (e.g. moving multiple files to trash) grows while the toast
    /// is still visible, so the user gets fresh time to undo.
    pub fn restart(&self, title: &str) {
        self.inner.title.replace(title.to_string());
        let mut active_ref = self.inner.active.borrow_mut();
        if let Some(active) = active_ref.as_mut() {
            active.label.set_text(title);
            let _ = remove_source_id(&active.timeout_id);
            active.timeout_id = self.start_timer(active.progress.clone());
        }
    }

    fn start_timer(&self, progress: ProgressBar) -> SourceId {
        let started = Instant::now();
        let timeout = Duration::from_secs(TOAST_TIMEOUT.into());
        progress.set_fraction(1.0);
        glib::timeout_add_local(
            TOAST_PROGRESS_INTERVAL,
            clone!(
                #[strong(rename_to=this)]
                self,
                #[strong]
                progress,
                move || {
                    let elapsed = started.elapsed();
                    if elapsed >= timeout {
                        this.dismiss();
                        ControlFlow::Break
                    } else {
                        let remaining = 1.0 - elapsed.as_secs_f64() / timeout.as_secs_f64();
                        progress.set_fraction(remaining);
                        ControlFlow::Continue
                    }
                }
            ),
        )
    }

    fn show(&self, overlay: &Overlay) {
        self.dismiss();

        let toast_box = GtkBox::new(Orientation::Vertical, 8);
        toast_box.add_css_class("toast");
        toast_box.set_valign(Align::End);

        let content_box = GtkBox::new(Orientation::Horizontal, 12);
        content_box.set_valign(Align::Center);
        let label = Label::new(Some(&self.inner.title.borrow()));
        content_box.append(&label);

        if let Some(button_label) = self.inner.button_label.as_ref() {
            let button = Button::with_label(button_label);
            if let Some(action_name) = self.inner.action_name.as_ref() {
                button.set_action_name(Some(action_name));
            }
            content_box.append(&button);
        }

        let progress = ProgressBar::new();
        progress.add_css_class("dialog-progress");
        progress.set_fraction(1.0);
        progress.set_hexpand(true);

        toast_box.append(&content_box);
        toast_box.append(&progress);

        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::SlideUp);
        revealer.set_transition_duration(TOAST_TRANSITION);
        revealer.set_child(Some(&toast_box));
        revealer.set_halign(Align::Center);
        revealer.set_valign(Align::End);
        revealer.set_margin_bottom(24);
        revealer.set_reveal_child(true);

        overlay.add_overlay(&revealer);

        let timeout_id = self.start_timer(progress.clone());

        self.inner.active.replace(Some(ActiveToast {
            overlay: overlay.clone(),
            revealer,
            timeout_id,
            label,
            progress,
        }));
    }
}
