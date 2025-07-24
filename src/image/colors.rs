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

use cairo::Context;

// generated file - do not edit

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    White,            // #ffffff
    Black,            // #000000
    Red,              // #ff0000
    BlackBean,        // #331a1a
    Gray,             // #808080
    Silver,           // #cccccc
    RussianViolet,    // #330033
    Citrine,          // #e6cc33
    FolderBack,       // #1e3319
    FolderTitle,      // #94bc8a
    FolderMsg,        // #c0cebf
    ArchiveBack,      // #23213c
    ArchiveTitle,     // #89a6d2
    ArchiveMsg,       // #c3ccd9
    UnsupportedBack,  // #292500
    UnsupportedTitle, // #c4b850
    UnsupportedMsg,   // #d0cc9f
    ErrorBack,        // #372020
    ErrorTitle,       // #b38888
    ErrorMsg,         // #dbb5b5
}

pub trait CairoColorExt {
    fn color(&self, color: Color);
}

impl CairoColorExt for Context {
    fn color(&self, color: Color) {
        match color {
            Color::White => {
                self.set_source_rgb(1.00, 1.00, 1.00); // #ffffff
            }
            Color::Black => {
                self.set_source_rgb(0.00, 0.00, 0.00); // #000000
            }
            Color::Red => {
                self.set_source_rgb(1.00, 0.00, 0.00); // #ff0000
            }
            Color::BlackBean => {
                self.set_source_rgb(0.20, 0.10, 0.10); // #331a1a
            }
            Color::Gray => {
                self.set_source_rgb(0.50, 0.50, 0.50); // #808080
            }
            Color::Silver => {
                self.set_source_rgb(0.80, 0.80, 0.80); // #cccccc
            }
            Color::RussianViolet => {
                self.set_source_rgb(0.20, 0.00, 0.20); // #330033
            }
            Color::Citrine => {
                self.set_source_rgb(0.90, 0.80, 0.20); // #e6cc33
            }
            Color::FolderBack => {
                self.set_source_rgb(0.12, 0.20, 0.10); // #1e3319
            }
            Color::FolderTitle => {
                self.set_source_rgb(0.58, 0.74, 0.54); // #94bc8a
            }
            Color::FolderMsg => {
                self.set_source_rgb(0.75, 0.81, 0.75); // #c0cebf
            }
            Color::ArchiveBack => {
                self.set_source_rgb(0.14, 0.13, 0.24); // #23213c
            }
            Color::ArchiveTitle => {
                self.set_source_rgb(0.54, 0.65, 0.82); // #89a6d2
            }
            Color::ArchiveMsg => {
                self.set_source_rgb(0.76, 0.80, 0.85); // #c3ccd9
            }
            Color::UnsupportedBack => {
                self.set_source_rgb(0.16, 0.15, 0.00); // #292500
            }
            Color::UnsupportedTitle => {
                self.set_source_rgb(0.77, 0.72, 0.31); // #c4b850
            }
            Color::UnsupportedMsg => {
                self.set_source_rgb(0.82, 0.80, 0.62); // #d0cc9f
            }
            Color::ErrorBack => {
                self.set_source_rgb(0.22, 0.13, 0.13); // #372020
            }
            Color::ErrorTitle => {
                self.set_source_rgb(0.70, 0.53, 0.53); // #b38888
            }
            Color::ErrorMsg => {
                self.set_source_rgb(0.86, 0.71, 0.71); // #dbb5b5
            }
        }
    }
}
