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

use std::path::{Path, PathBuf};

use glib::{ffi::g_source_remove, object::IsA, result_from_gboolean, BoolError, SourceId};
use gtk4::{
    prelude::{BoxExt, ButtonExt, GtkWindowExt, WidgetExt},
    Orientation, Window,
};
use sha2::{Digest, Sha256};

/// Safer alternative to SourceId::remove()
pub fn remove_source_id(id: &SourceId) -> Result<(), BoolError> {
    unsafe { result_from_gboolean!(g_source_remove(id.as_raw()), "Failed to remove source") }
}

pub fn path_to_filename<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

pub fn path_to_directory<P: AsRef<Path>>(path: P) -> String {
    match path.as_ref().parent() {
        Some(path) => path.to_string_lossy().to_string(),
        None => Default::default(),
    }
}

pub fn path_to_extension<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase()
}

pub fn ellipsis_middle(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }

    if max_len < 4 {
        // If max_len is too small for ellipses, just truncate
        return s.chars().take(max_len).collect();
    }

    let available_len = max_len - 3;
    let start_len = available_len.div_ceil(2); // Round up for start
    let end_len = available_len / 2; // Round down for end

    let start: String = s.chars().take(start_len).collect();
    let end: String = s.chars().skip(s.chars().count() - end_len).collect();

    format!("{}...{}", start, end)
}

pub fn mview_hash(path: &Path, extra: Option<&str>, extension: &str) -> PathBuf {
    let mut hasher = Sha256::new();
    if let Some(filename) = path.file_name() {
        hasher.update(filename.to_string_lossy().to_string().as_bytes());
    }
    if let Some(extra) = extra {
        hasher.update(extra.as_bytes());
    }
    let sha256sum = format!("{:x}", hasher.finalize());
    let thumb_filename = format!("{sha256sum}.{extension}");
    if let Some(parent) = path.parent() {
        parent.join(".mview").join(thumb_filename)
    } else {
        Path::new(".mview").join(thumb_filename)
    }
}

pub fn show_error_dialog(window: &impl IsA<gtk4::Window>, title: &str, message: &str) {
    let dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(format!("<b>{}</b>", title))
        .detail(message)
        .buttons(["OK"])
        .default_button(0)
        .cancel_button(0)
        .build();
    dialog.show(Some(window));
}

/// A small modal, non-closable dialog with a message and progress bar, used to
/// give feedback to the user during a background operation (e.g. preview
/// creation). Call [`ProgressDialog::set_progress`] to update the displayed
/// step count and bar fraction, and [`ProgressDialog::close`] once the
/// operation has finished.
pub struct ProgressDialog {
    window: gtk4::Window,
    progress_bar: gtk4::ProgressBar,
}

/// Progress messages sent from the preview-creation background thread to the
/// GTK main loop.
pub enum PreviewProgress {
    Step(u32, u32),
    Done(crate::error::MviewResult<()>),
}

impl ProgressDialog {
    pub fn new(parent: &impl IsA<gtk4::Window>, title: &str, message: &str) -> Self {
        let label = gtk4::Label::new(Some(message));
        label.set_wrap(true);

        let progress_bar = gtk4::ProgressBar::new();
        progress_bar.add_css_class("dialog-progress");
        progress_bar.set_show_text(true);
        progress_bar.set_text(Some("Starting..."));
        progress_bar.set_fraction(0.0);
        progress_bar.set_hexpand(true);
        progress_bar.set_width_request(280);

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        content.set_margin_top(20);
        content.set_margin_bottom(20);
        content.set_margin_start(20);
        content.set_margin_end(20);
        content.append(&label);
        content.append(&progress_bar);

        let window = gtk4::Window::builder()
            .transient_for(parent)
            .modal(true)
            .deletable(false)
            .resizable(false)
            .title(title)
            .child(&content)
            .build();
        window.present();

        Self {
            window,
            progress_bar,
        }
    }

    /// Updates the progress bar's fraction and its "Step n of m" text.
    pub fn set_progress(&self, current: u32, total: u32) {
        let fraction = if total == 0 {
            0.0
        } else {
            current as f64 / total as f64
        };
        self.progress_bar.set_fraction(fraction.clamp(0.0, 1.0));
        self.progress_bar
            .set_text(Some(&format!("Step {current} of {total}")));
    }

    pub fn close(&self) {
        self.window.close();
    }
}

pub fn error_dialog(parent_window: &impl IsA<gtk4::Window>, title: &str, message: &str) {
    // 1. Create a standard Window styled to look like a modal dialog
    let dialog = Window::builder()
        .title(title)
        .transient_for(parent_window)
        .modal(true)
        .destroy_with_parent(true)
        .default_width(300)
        .build();

    // 2. Build the Layout Container
    let content_box = gtk4::Box::new(Orientation::Vertical, 12);
    content_box.set_margin_top(18);
    content_box.set_margin_bottom(18);
    content_box.set_margin_start(18);
    content_box.set_margin_end(18);

    let message_label = gtk4::Label::new(Some(message));
    message_label.add_css_class("error-dialog-message"); // Target this in CSS
    message_label.set_halign(gtk4::Align::Start);

    // 4. Create the OK Action Button
    let ok_button = gtk4::Button::with_label("OK");
    ok_button.set_halign(gtk4::Align::End);

    // Connect the button to close the window when clicked
    let dialog_clone = dialog.clone();
    ok_button.connect_clicked(move |_| {
        dialog_clone.destroy();
    });

    // 5. Assemble the UI Hierarchy
    content_box.append(&message_label);
    content_box.append(&ok_button);
    dialog.set_child(Some(&content_box));

    // 6. Render it on-screen
    dialog.present();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_string_middle() {
        assert_eq!(ellipsis_middle("Hello", 0), "");
        assert_eq!(ellipsis_middle("Hello", 1), "H");
        assert_eq!(ellipsis_middle("Hello", 2), "He");
        assert_eq!(ellipsis_middle("Hello", 3), "Hel");
        assert_eq!(ellipsis_middle("Hello", 4), "H...");
        assert_eq!(ellipsis_middle("Hello", 5), "Hello");
        assert_eq!(ellipsis_middle("Hello", 6), "Hello");
        assert_eq!(ellipsis_middle("Hello, World!", 9), "Hel...ld!");
        assert_eq!(ellipsis_middle("Hello, World!", 10), "Hell...ld!");
        assert_eq!(ellipsis_middle("Hello, World!", 11), "Hell...rld!");
        assert_eq!(ellipsis_middle("Hello, World!", 12), "Hello...rld!");
        assert_eq!(ellipsis_middle("Hello, World!", 13), "Hello, World!");
        assert_eq!(ellipsis_middle("", 5), "");
    }

    #[test]
    fn test_hash() {
        assert_eq!(mview_hash(Path::new("/some/dir/foo.jpg"), None, "ext1"), Path::new("/some/dir/.mview/e29273cf02c3670bdf0e242cb77874b4083565430ac9c44fa0f10847638a69fd.ext1"));
        assert_eq!(mview_hash(Path::new("/some/dir/foo.jpg"), Some("bar"), "ext2"), Path::new("/some/dir/.mview/77901352b49ea1483d8b4216a84517895e7b6fe263ca55da0d847525f34f4f94.ext2"));
        assert_eq!(
            mview_hash(Path::new("foo.jpg"), None, "ext3"),
            Path::new(
                ".mview/e29273cf02c3670bdf0e242cb77874b4083565430ac9c44fa0f10847638a69fd.ext3"
            )
        );
    }
}
