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

use resvg::usvg::Tree;

use crate::{
    backends::document::PageMode,
    file_view::model::Reference,
    image::{provider::surface::SurfaceData, view::Zoom},
    rect::RectD,
};

#[derive(Debug, Clone)]
pub enum RenderCommand {
    // Image((Reference, PageMode, i32)),
    RenderDoc(Reference, u32, PageMode, Zoom, RectD),
    RenderSvg(u32, Zoom, RectD, Box<Tree>),
}

#[derive(Debug, Clone)]
pub struct RenderCommandMessage {
    pub id: u32,
    pub cmd: RenderCommand,
}

#[derive(Debug, Clone)]
pub enum RenderReply {
    // Image((Reference, PageMode, i32)),
    RenderDone(u32, SurfaceData, Zoom),
}

#[derive(Debug, Clone)]
pub struct RenderReplyMessage {
    pub _id: u32,
    pub reply: RenderReply,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn test_send_sync() {
        assert_send_sync::<RenderCommandMessage>();
        assert_send_sync::<RenderReplyMessage>();
        assert_send_sync::<SurfaceData>();
    }
}
