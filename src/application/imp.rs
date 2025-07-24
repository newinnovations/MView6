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

use std::cell::OnceCell;

use gio::subclass::prelude::{ApplicationImpl, ApplicationImplExt};
use glib::subclass::{
    object::ObjectImpl,
    types::{ObjectSubclass, ObjectSubclassExt},
};
use gtk4::{glib, prelude::GtkWindowExt, subclass::prelude::GtkApplicationImpl, Application};

use crate::window::MViewWindow;

#[derive(Debug, Default)]
pub struct MviewApplicationImp {
    window: OnceCell<MViewWindow>,
}

#[glib::object_subclass]
impl ObjectSubclass for MviewApplicationImp {
    const NAME: &'static str = "MviewApplication";
    type Type = super::MviewApplication;
    type ParentType = Application;
}

impl ObjectImpl for MviewApplicationImp {}

/// When our application starts, the `startup` signal will be fired.
/// This gives us a chance to perform initialisation tasks that are not directly
/// related to showing a new window. After this, depending on how
/// the application is started, either `activate` or `open` will be called next.
impl ApplicationImpl for MviewApplicationImp {
    fn startup(&self) {
        self.parent_startup();
        let window = MViewWindow::new(&self.obj());
        window.present();
        self.window
            .set(window)
            .expect("Failed to initialize application window");
    }
}

impl GtkApplicationImpl for MviewApplicationImp {}
