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

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use async_channel::{Receiver, Sender};

use crate::{
    backends::Backend,
    file_view::BackendRef,
    image::render_svg,
    render_thread::model::{RenderCommand, RenderCommandMessage, RenderReply, RenderReplyMessage},
};

#[derive(Debug, Clone)]
pub struct RenderWorker {
    to_rt_receiver: Receiver<RenderCommandMessage>,
    from_rt_sender: Sender<RenderReplyMessage>,
    command_id: Arc<AtomicU64>,
}

impl RenderWorker {
    pub fn new(
        from_rt_sender: Sender<RenderReplyMessage>,
        to_rt_receiver: Receiver<RenderCommandMessage>,
        counter: Arc<AtomicU64>,
    ) -> Self {
        RenderWorker {
            to_rt_receiver,
            from_rt_sender,
            command_id: counter,
        }
    }

    pub fn run(&self) {
        let mut backend = <dyn Backend>::none();
        let mut backend_ref = BackendRef::None;
        loop {
            // recv_blocking blocks until a message arrives or all senders are dropped.
            let command = match self.to_rt_receiver.recv_blocking() {
                Ok(cmd) => cmd,
                Err(_) => {
                    // All senders have been dropped; exit cleanly.
                    break;
                }
            };

            match command.cmd {
                RenderCommand::Shutdown => break,
                RenderCommand::RenderDoc(image_id, zoom, viewport, doc) => {
                    if self.get_current_command_id() != command.id {
                        println!(
                            "There are newer commands in the queue, skipping id {}",
                            command.id
                        );
                        continue;
                    }

                    if doc.reference.backend != backend_ref {
                        println!("Changing backend to {:?}", doc.reference.backend);
                        match <dyn Backend>::new_from_ref(&doc.reference.backend) {
                            Ok(new_backend) => backend = new_backend,
                            Err(e) => {
                                eprintln!(
                                    "Failed to create backend for {:?}: {e}",
                                    doc.reference.backend
                                );
                                // backend = <dyn Backend>::none();
                                continue;
                            }
                        }
                        backend_ref = doc.reference.backend;
                    }
                    // NOTE: mid-flight cancellation is not supported; the render call is
                    // synchronous and opaque to us. Freshness is re-checked after completion.
                    let result =
                        backend.render(&doc.reference.item, &doc.page_mode, &zoom, &viewport);
                    if let Some(surface) = result {
                        if command.id != self.get_current_command_id() {
                            println!(
                                "Result from hq render not needed anymore. Discarding id {}",
                                command.id
                            );
                            continue;
                        }
                        let reply = RenderReplyMessage {
                            id: command.id,
                            reply: RenderReply::RenderDone(image_id, surface, zoom, viewport),
                        };
                        if let Err(e) = self.from_rt_sender.send_blocking(reply) {
                            eprintln!("Failed to send reply {e}");
                        }
                    } else {
                        println!("HqRender: none");
                    }
                }
                RenderCommand::RenderSvg(image_id, zoom, viewport, tree) => {
                    if self.get_current_command_id() != command.id {
                        println!(
                            "There are newer commands in the queue, skipping id {}",
                            command.id
                        );
                        continue;
                    }

                    // NOTE: mid-flight cancellation is not supported (see RenderDoc comment).
                    let result = render_svg(&zoom, &viewport, &tree);
                    if let Some(surface) = result {
                        if command.id != self.get_current_command_id() {
                            println!(
                                "Result from svg render not needed anymore. Discarding id {}",
                                command.id
                            );
                            continue;
                        }
                        let reply = RenderReplyMessage {
                            id: command.id,
                            reply: RenderReply::RenderDone(image_id, surface, zoom, viewport),
                        };
                        if let Err(e) = self.from_rt_sender.send_blocking(reply) {
                            eprintln!("Failed to send reply {e}");
                        }
                    } else {
                        println!("HqRender: none");
                    }
                }
            }
        }
    }

    fn get_current_command_id(&self) -> u64 {
        self.command_id.load(Ordering::SeqCst)
    }
}
