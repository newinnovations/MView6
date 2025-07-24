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

use std::time::SystemTime;

use image::DynamicImage;

use crate::{
    backends::{
        archive_mar::TMarReference, archive_rar::TRarReference, archive_zip::TZipReference,
        document::mupdf::TDocReference, filesystem::TFileReference, Backend,
    },
    category::Category,
    file_view::Target,
    image::colors::Color,
};

pub struct TParent {
    pub backend: Box<dyn Backend>,
    pub target: Target,
    pub focus_pos: i32,
}

#[derive(Debug, Clone)]
pub enum TReference {
    FileReference(TFileReference),
    ZipReference(TZipReference),
    MarReference(TMarReference),
    RarReference(TRarReference),
    DocReference(TDocReference),
    None,
}

#[derive(Debug, Clone)]
pub struct TEntry {
    pub category: Category,
    pub name: String,
    pub reference: TReference,
}

impl TEntry {
    pub fn new(category: Category, name: &str, reference: TReference) -> Self {
        TEntry {
            category,
            name: name.to_string(),
            reference,
        }
    }
}

impl Default for TEntry {
    fn default() -> Self {
        Self {
            category: Category::Unsupported,
            name: Default::default(),
            reference: TReference::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TCommand {
    pub id: u32,
    pub page: i32,
    pub start: SystemTime,
    pub tasks: Vec<TTask>,
    pub todo: usize,
    pub last_update: f64,
    pub dim: SheetDimensions,
}

impl Default for TCommand {
    fn default() -> Self {
        Self {
            id: Default::default(),
            page: Default::default(),
            start: SystemTime::now(),
            tasks: Default::default(),
            todo: 0,
            last_update: 0.0,
            dim: Default::default(),
        }
    }
}

impl TCommand {
    pub fn new(id: u32, page: i32, tasks: Vec<TTask>, dim: SheetDimensions) -> Self {
        let todo = tasks.len();
        TCommand {
            id,
            page,
            start: SystemTime::now(),
            tasks,
            todo,
            last_update: 0.0,
            dim,
        }
    }

    pub fn elapsed(&self) -> f64 {
        if let Ok(elapsed) = self.start.elapsed() {
            elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9
        } else {
            0.0
        }
    }

    pub fn needs_work(&self) -> bool {
        self.todo != 0
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TTask {
    pub id: i32,
    pub size: u32,
    pub position: (i32, i32),
    pub source: TEntry,
    pub annotation: Annotation,
}

impl TTask {
    pub fn new(id: i32, size: u32, x: i32, y: i32, source: TEntry, annotation: Annotation) -> Self {
        TTask {
            id,
            size,
            position: (x, y),
            source,
            annotation,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TMessage {
    pub title: String,
    pub message: String,
    pub colors: (Color, Color, Color),
}

impl TMessage {
    pub fn new(title: &str, message: &str, colors: (Color, Color, Color)) -> Self {
        TMessage {
            title: title.to_string(),
            message: message.to_string(),
            colors,
        }
    }
    pub fn error(title: &str, message: &str) -> Self {
        TMessage {
            title: title.to_string(),
            message: message.to_string(),
            colors: (Color::ErrorBack, Color::ErrorTitle, Color::ErrorMsg),
        }
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone)]
pub enum TResultOption {
    Image(DynamicImage),
    Message(TMessage),
}

#[derive(Debug, Clone)]
pub struct TResult {
    pub id: u32,
    pub task: TTask,
    pub result: TResultOption,
}

impl TResult {
    pub fn new(id: u32, task: TTask, result: TResultOption) -> Self {
        TResult { id, task, result }
    }
}

pub enum Message {
    Command(Box<TCommand>),
    Result(Box<TResult>),
}

#[derive(Default, Debug, Clone)]
pub struct SheetDimensions {
    pub size: i32,
    pub width: i32,
    pub height: i32,
    pub separator_x: i32,
    pub separator_y: i32,
    pub capacity_x: i32,
    pub capacity_y: i32,
    pub offset_x: i32,
    pub offset_y: i32,
}

impl SheetDimensions {
    pub fn capacity(&self) -> i32 {
        self.capacity_x * self.capacity_y
    }

    pub fn rel_position(&self, x: f64, y: f64) -> Option<i32> {
        let x = (x as i32 - self.offset_x) / (self.size + self.separator_x);
        let y = (y as i32 - self.offset_y) / (self.size + self.separator_y);
        if x < 0 || y < 0 || x >= self.capacity_x || y >= self.capacity_y {
            None
        } else {
            Some(y * self.capacity_x + x)
        }
    }

    pub fn abs_position(&self, page: i32, x: f64, y: f64) -> Option<i32> {
        self.rel_position(x, y)
            .map(|rel| page * self.capacity() + rel)
    }
}

#[derive(Debug, Clone)]
pub struct Annotations {
    pub dim: SheetDimensions,
    pub page: i32,
    pub annotations: Vec<Annotation>,
}

impl Annotations {
    pub fn get(&self, index: Option<i32>) -> Option<&Annotation> {
        self.annotations.get(index? as usize)
    }

    // pub fn get_at(&self, x: f64, y: f64) -> Option<&Annotation> {
    //     self.get(self.dim.rel_position(x, y))
    //         .filter(|a| a.position.inside(x, y))
    // }

    pub fn index_at(&self, x: f64, y: f64) -> Option<i32> {
        let index = self.dim.rel_position(x, y)?;
        let annotation = self.annotations.get(index as usize)?;
        annotation.position.inside(x, y).then_some(index)
    }
}

#[derive(Debug, Clone)]
pub struct TRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl TRect {
    pub fn new_i32(x: i32, y: i32, width: i32, height: i32) -> Self {
        TRect {
            x: x as f64,
            y: y as f64,
            width: width as f64,
            height: height as f64,
        }
    }

    pub fn inside(&self, x: f64, y: f64) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}

#[derive(Debug, Clone)]
pub struct Annotation {
    pub id: i32,
    pub position: TRect,
    pub name: String,
    pub category: Category,
    pub reference: TReference,
}

impl PartialEq for Annotation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
