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

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use async_channel::Sender;

use crate::render_thread::model::{RenderCommand, RenderCommandMessage};

#[derive(Debug, Clone)]
pub struct RenderThreadSender {
    sender: Sender<RenderCommandMessage>,
    counter: Arc<AtomicU32>,
}

impl RenderThreadSender {
    pub fn new(sender: Sender<RenderCommandMessage>, counter: Arc<AtomicU32>) -> Self {
        Self { sender, counter }
    }

    pub fn send_blocking(&self, command: RenderCommand) {
        let id = 1 + self.counter.load(Ordering::SeqCst);
        self.counter.store(id, Ordering::SeqCst);
        let msg = RenderCommandMessage { id, cmd: command };
        let _ = self.sender.send_blocking(msg);
    }
}
