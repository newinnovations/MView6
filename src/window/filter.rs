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

use gtk4::{
    prelude::{BoxExt, CheckButtonExt, DialogExt, GtkWindowExt},
    Box, CheckButton, Dialog, Orientation, ResponseType,
};

use crate::window::MViewWindow;

pub fn create_filter_dialog(parent: &MViewWindow) -> Dialog {
    let dialog = Dialog::builder()
        .title("Select Items")
        .modal(true)
        .transient_for(parent)
        .build();

    let content_area = dialog.content_area();
    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(12)
        .margin_start(12)
        .margin_end(12)
        .margin_top(12)
        .margin_bottom(12)
        .build();

    let items = vec!["Item 1", "Item 2", "Item 3", "Item 4", "Item 5"];
    let mut checkboxes = Vec::new();

    for item in items {
        let checkbox = CheckButton::with_label(item);
        vbox.append(&checkbox);
        checkboxes.push(checkbox);
    }

    content_area.append(&vbox);
    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button("OK", ResponseType::Ok);

    dialog.connect_response(move |dialog, response| {
        // if response == ResponseType::Ok {
        //     let selected: Vec<String> = checkboxes
        //         .iter()
        //         .filter(|cb| cb.is_active())
        //         .filter_map(|cb| cb.label().map(|s| s.to_string()))
        //         .collect();

        //     let _ = sender.send(Some(selected));
        // } else {
        //     let _ = sender.send(None);
        // }
        dialog.close();
    });

    dialog
}
