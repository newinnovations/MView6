// MView6 -- Opiniated image browser written in Rust and GTK4
//
// Copyright (c) 2024 Martin van der Werff <github (at) newinnovations.nl>
//
// This file is part of MView6.
//
// MView6 is free software: you can redistribute it and/or modify it under the terms of
// the GNU General Public License as published by the Free Software Foundation, either version 3
// of the License, or (at your option) any later version.
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
        archive_rar::TRarReference, archive_zip::TZipReference, filesystem::TFileReference,
        pdf::TPdfReference,
    },
    category::Category,
    image::colors::Color,
};

#[derive(Debug, Clone)]
pub enum TReference {
    FileReference(TFileReference),
    ZipReference(TZipReference),
    RarReference(TRarReference),
    PdfReference(TPdfReference),
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
    pub start: SystemTime,
    pub tasks: Vec<TTask>,
    pub todo: usize,
    pub last_update: f64,
}

impl Default for TCommand {
    fn default() -> Self {
        Self {
            id: Default::default(),
            start: SystemTime::now(),
            tasks: Default::default(),
            todo: 0,
            last_update: 0.0,
        }
    }
}

impl TCommand {
    pub fn new(id: u32, tasks: Vec<TTask>) -> Self {
        let todo = tasks.len();
        TCommand {
            id,
            start: SystemTime::now(),
            tasks,
            todo,
            last_update: 0.0,
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
    pub tid: i32,
    pub size: u32,
    pub position: (i32, i32),
    pub source: TEntry,
}

impl TTask {
    pub fn new(tid: i32, size: u32, x: i32, y: i32, source: TEntry) -> Self {
        TTask {
            tid,
            size,
            position: (x, y),
            source,
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
    Command(TCommand),
    Result(TResult),
}
