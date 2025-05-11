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

#![windows_subsystem = "windows"]

mod application;
mod backends;
mod category;
mod config;
mod error;
mod file_view;
mod image;
mod info_view;
mod profile;
mod window;

use gtk4::{
    gdk::Display, prelude::ApplicationExtManual, style_context_add_provider_for_display,
    CssProvider, IconTheme, STYLE_PROVIDER_PRIORITY_APPLICATION,
};
use std::env;

fn main() {
    gtk4::init().expect("Failed to initialize gtk");

    let args: Vec<String> = env::args().collect();
    let filename = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };
    dbg!(filename);

    gio::resources_register_include!("mview6.gresource").unwrap();

    let display = Display::default().expect("Could not connect to a display.");

    let css_provider = CssProvider::new();
    css_provider.load_from_resource("/css/mview6.css");
    style_context_add_provider_for_display(
        &display,
        &css_provider,
        STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let icon_theme = IconTheme::for_display(&display);
    icon_theme.add_resource_path("/icons");

    let app = application::MviewApplication::new();

    app.run();
}
