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

#![windows_subsystem = "windows"]

mod application;
mod backends;
mod classification;
mod config;
mod content;
mod error;
mod file_view;
mod image;
mod info_view;
mod profile;
mod rect;
mod render_thread;
mod util;
mod window;

use std::sync::OnceLock;

pub use error::AppError;
pub use error::MviewError;

use clap::{Parser, ValueEnum};
use gtk4::{
    gdk::Display, prelude::ApplicationExtManual, style_context_add_provider_for_display,
    CssProvider, IconTheme, STYLE_PROVIDER_PRIORITY_APPLICATION,
};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum SortOptions {
    /// Sort by file type ascending
    #[value(name = "0a")]
    TypeAscending,
    /// Sort by file type descending
    #[value(name = "0d")]
    TypeDescending,
    /// Sort by name ascending
    #[value(name = "1a")]
    NameAscending,
    /// Sort by name descending
    #[value(name = "1d")]
    NameDescending,
    /// Sort by size ascending
    #[value(name = "2a")]
    SizeAscending,
    /// Sort by size descending
    #[value(name = "2d")]
    SizeDescending,
    /// Sort by date ascending
    #[value(name = "3a")]
    DateAscending,
    /// Sort by date descending
    #[value(name = "3d")]
    DateDescending,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum FilterOptions {
    /// Show all files
    All,
    /// Show only images
    Image,
    /// Show only videos
    Video,
    /// Show only documents
    Document,
    /// Show only archives
    Archive,
}

#[derive(Parser, Debug, Clone)]
#[command(
    version,
    about = "MView6 - High-performance PDF and photo viewer built with Rust and GTK4"
)]
pub struct Args {
    /// File or directory to open
    #[arg(value_name = "FILE OR DIRECTORY", value_hint = clap::ValueHint::FilePath)]
    filename: Option<String>,

    #[arg(short, long, value_enum, default_value_t = SortOptions::TypeAscending)]
    sort: SortOptions,

    #[arg(short, long, value_enum, default_value_t = FilterOptions::All)]
    filter: FilterOptions,
}

pub static ARGS: OnceLock<Args> = OnceLock::new();

fn main() {
    ARGS.set(Args::parse()).unwrap();

    gtk4::init().expect("Failed to initialize gtk");

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

    pdfium::set_library_location("/usr/lib/mview6");

    let app = application::MviewApplication::new();

    app.run();
}
