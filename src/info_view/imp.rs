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

use glib::subclass::{
    object::{ObjectImpl, ObjectImplExt},
    types::{ObjectSubclass, ObjectSubclassExt},
};
use gtk4::{
    glib,
    pango::WrapMode,
    prelude::{CellRendererExt, CellRendererTextExt, TreeViewExt},
    subclass::{prelude::TreeViewImpl, widget::WidgetImpl},
    CellRendererText, TreeView, TreeViewColumn, TreeViewColumnSizing,
};

use super::{Columns, InfoView};

#[derive(Debug, Default)]
pub struct InfoViewImp {}

#[glib::object_subclass]
impl ObjectSubclass for InfoViewImp {
    const NAME: &'static str = "InfoView";
    type Type = InfoView;
    type ParentType = TreeView;
}

const WIDTH_KEY: i32 = 110;
const WIDTH_VALUE: i32 = 210;
const PADDING_X: i32 = 2;
const PADDING_Y: i32 = 3;

impl ObjectImpl for InfoViewImp {
    fn constructed(&self) {
        self.parent_constructed();
        let instance = self.obj();

        let renderer_txt = CellRendererText::new();
        renderer_txt.set_padding(PADDING_X, PADDING_Y);
        renderer_txt.set_wrap_mode(WrapMode::WordChar);
        renderer_txt.set_wrap_width(WIDTH_KEY);
        renderer_txt.set_yalign(0.0f32);
        let col_key = TreeViewColumn::new();
        col_key.pack_start(&renderer_txt, true);
        col_key.set_title("Key");
        col_key.add_attribute(&renderer_txt, "text", Columns::Key as i32);
        col_key.set_sizing(TreeViewColumnSizing::Fixed);
        col_key.set_fixed_width(WIDTH_KEY);
        col_key.set_sort_column_id(Columns::Key as i32);
        instance.append_column(&col_key);

        let renderer_txt = CellRendererText::new();
        renderer_txt.set_wrap_mode(WrapMode::WordChar);
        renderer_txt.set_wrap_width(WIDTH_VALUE);
        renderer_txt.set_yalign(0.0f32);
        renderer_txt.set_padding(PADDING_X, PADDING_Y);
        let col_value = TreeViewColumn::new();
        col_value.pack_start(&renderer_txt, true);
        col_value.set_title("Value");
        col_value.add_attribute(&renderer_txt, "text", Columns::Value as i32);
        col_value.set_sizing(TreeViewColumnSizing::Fixed);
        col_value.set_fixed_width(WIDTH_VALUE);
        col_value.set_sort_column_id(Columns::Value as i32);
        instance.append_column(&col_value);
    }
}

impl WidgetImpl for InfoViewImp {}

impl TreeViewImpl for InfoViewImp {}

impl InfoViewImp {}
