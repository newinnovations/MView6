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

use gtk4::prelude::*;
use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType};
use std::env;
use std::path::Path;

use crate::window::MViewWindow;

pub fn check_dependencies(parent_window: &MViewWindow, show_success: bool) {
    let required_fonts = vec![
        "CascadiaMono-Regular.ttf",
        "LiberationSans-Bold.ttf",
        "LiberationSans-Regular.ttf",
    ];

    let pdfium_lib = if cfg!(target_os = "windows") {
        "pdfium.dll"
    } else if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else {
        "libpdfium.so"
    };

    let install_path = get_install_path();
    let mut missing_files = Vec::new();

    // Check for font files
    for font in &required_fonts {
        let font_path = install_path.join(font);
        if !font_path.exists() {
            missing_files.push(font.to_string());
        }
    }

    // Check for PDFium library
    let pdfium_path = install_path.join(pdfium_lib);
    if !pdfium_path.exists() {
        missing_files.push(pdfium_lib.to_string());
    }

    if missing_files.is_empty() {
        if show_success {
            show_success_dialog(parent_window);
        }
    } else {
        show_missing_files_dialog(parent_window, &missing_files, &install_path);
    }
}

fn get_install_path() -> std::path::PathBuf {
    if cfg!(target_os = "windows") {
        // On Windows, check the current executable directory
        match env::current_exe() {
            Ok(exe_path) => {
                if let Some(parent) = exe_path.parent() {
                    parent.to_path_buf()
                } else {
                    std::path::PathBuf::from(".")
                }
            }
            Err(_) => std::path::PathBuf::from("."),
        }
    } else {
        // On Linux/Unix, use /usr/lib/mview6
        std::path::PathBuf::from("/usr/lib/mview6")
    }
}

fn show_success_dialog(parent_window: &MViewWindow) {
    let dialog = MessageDialog::new(
        Some(parent_window),
        DialogFlags::MODAL,
        MessageType::Info,
        ButtonsType::Ok,
        "All Required Files Found!",
    );

    dialog.set_secondary_text(Some(
        "All MView6 dependencies are properly installed and ready to use.",
    ));

    dialog.connect_response(|dialog, _| {
        dialog.close();
    });

    dialog.show();
}

fn show_missing_files_dialog(
    parent_window: &MViewWindow,
    missing_files: &[String],
    install_path: &Path,
) {
    let dialog = MessageDialog::new(
        Some(parent_window),
        DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        "Missing MView6 Dependencies",
    );

    // Set a wider dialog
    dialog.set_default_size(600, 400);

    let os_specific_instructions = if cfg!(target_os = "windows") {
        format!(
            "<b>Missing files:</b>\n - <tt>{}</tt>\n\n\
            <b>To fix this issue:</b>\n\n\
            <b>1.</b> Download the font files from:\n   \
            <span color='lightgreen'><u>https://github.com/newinnovations/mview6/tree/main/resources/fonts</u></span>\n\n\
            <b>2.</b> Download PDFium library from:\n   \
            <span color='lightgreen'><u>https://github.com/bblanchon/pdfium-binaries/releases</u></span>\n   \
            <i>(Download the Windows version: <tt>pdfium.dll</tt>)</i>\n\n\
            <b>3.</b> Copy all files to the same directory as the MView6 executable:\n   \
            <tt><span bgcolor='lightgray'>{}</span></tt>\n\n\
            <b>Important:</b> The missing files should be placed directly in this folder.",
            missing_files.join("</tt>,\n - <tt>"),
            install_path.display()
        )
    } else {
        format!(
            "<b>Missing files:</b>\n - <tt>{}</tt>\n\n\
            <b>To fix this issue:</b>\n\n\
            <b>1.</b> Create the installation directory (if it doesn't exist):\n   \
            <tt>sudo mkdir -p /usr/lib/mview6</tt>\n\n\
            <b>2.</b> Download the font files from:\n   \
            <span color='lightgreen'><u>https://github.com/newinnovations/mview6/tree/main/resources/fonts</u></span>\n\n\
            <b>3.</b> Download PDFium library from:\n   \
            <span color='lightgreen'><u>https://github.com/bblanchon/pdfium-binaries/releases</u></span>\n   \
            <i>(Download the Linux version: <tt>libpdfium.so</tt>)</i>\n\n\
            <b>4.</b> Copy all files to /usr/lib/mview6:\n   \
            <tt>sudo cp &lt;downloaded-files&gt; /usr/lib/mview6/</tt>\n\n\
            <b>5.</b> Ensure proper permissions:\n   \
            <tt>sudo chmod 644 /usr/lib/mview6/*</tt>\n\n\
            <b>Note:</b> You may need administrator privileges for these operations.",
            missing_files.join("</tt>,\n - <tt>")
        )
    };

    dialog.set_secondary_text(Some(&os_specific_instructions));
    dialog.set_secondary_use_markup(true);

    dialog.connect_response(|dialog, _| {
        dialog.close();
    });

    dialog.show();
}
