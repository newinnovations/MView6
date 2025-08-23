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

pub mod model;
mod sender;
mod worker;

use std::{
    sync::{atomic::AtomicU32, Arc},
    thread::{self, JoinHandle},
};

use async_channel::{Receiver, Sender};

use crate::render_thread::{
    model::{RenderCommandMessage, RenderReplyMessage},
    worker::RenderWorker,
};

pub use sender::RenderThreadSender;

#[derive(Debug)]
pub struct RenderThread {
    _handle: JoinHandle<()>,
    counter: Arc<AtomicU32>,
}

impl RenderThread {
    pub fn new(
        from_rt_sender: Sender<RenderReplyMessage>,
        to_rt_receiver: Receiver<RenderCommandMessage>,
    ) -> Self {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);
        let worker = RenderWorker::new(from_rt_sender, to_rt_receiver, counter_clone);
        let handle = thread::spawn(move || {
            worker.run();
        });
        RenderThread {
            _handle: handle,
            counter,
        }
    }

    pub fn create_sender(&self, to_rt_sender: Sender<RenderCommandMessage>) -> RenderThreadSender {
        RenderThreadSender::new(to_rt_sender, self.counter.clone())
    }
}
