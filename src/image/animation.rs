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

use std::{
    fs::File,
    io::{BufRead, BufReader, Cursor, Seek},
    time::{Duration, SystemTime},
};

use gdk_pixbuf::{Pixbuf, PixbufAnimationIter};
use image_webp::WebPDecoder;

use crate::error::MviewResult;

use super::{provider::webp::WebP, Image, ImageData};

#[derive(Default)]
pub enum Animation {
    #[default]
    None,
    Gdk(PixbufAnimationIter),
    WebPFile(Box<WebPAnimation<BufReader<File>>>),
    WebPMemory(Box<WebPAnimation<Cursor<Vec<u8>>>>),
}

pub(super) struct AnimationFrame {
    pub(super) delay_ms: u32,
    pub(super) pixbuf: Pixbuf,
}

pub struct WebPAnimation<T> {
    pub(super) decoder: WebPDecoder<T>,
    pub(super) index: u32,
    pub(super) first_run: bool,
    pub(super) frames: Vec<AnimationFrame>,
}

impl Image {
    pub fn is_animation(&self) -> bool {
        !matches!(self.animation, Animation::None)
    }

    pub fn animation_delay_time(&self, ts_previous_cb: SystemTime) -> Option<std::time::Duration> {
        match &self.animation {
            Animation::None => None,
            Animation::Gdk(animation) => animation.delay_time(),
            Animation::WebPFile(animation) => animation.delay_time(ts_previous_cb),
            Animation::WebPMemory(animation) => animation.delay_time(ts_previous_cb),
        }
    }

    pub fn animation_advance(&mut self, current_time: SystemTime) -> bool {
        match &mut self.animation {
            Animation::None => false,
            Animation::Gdk(animation) => {
                if animation.advance(current_time) {
                    self.image_data = ImageData::Single(animation.pixbuf());
                    true
                } else {
                    false
                }
            }
            Animation::WebPFile(animation) => match animation.advance(current_time) {
                Some(pixbuf) => {
                    self.image_data = ImageData::Single(pixbuf);
                    true
                }
                None => false,
            },
            Animation::WebPMemory(animation) => match animation.advance(current_time) {
                Some(pixbuf) => {
                    self.image_data = ImageData::Single(pixbuf);
                    true
                }
                None => false,
            },
        }
    }
}

impl<T: BufRead + Seek> WebPAnimation<T> {
    pub fn new(mut decoder: WebPDecoder<T>) -> MviewResult<Self> {
        let (pixbuf, delay_ms) = WebP::read_frame(&mut decoder)?;
        Ok(Self {
            decoder,
            index: 0,
            first_run: true,
            frames: vec![AnimationFrame { delay_ms, pixbuf }],
        })
    }

    fn delay_time(&self, ts_previous_cb: SystemTime) -> Option<Duration> {
        if let Some(frame) = self.frames.get(self.index as usize) {
            let interval = Duration::from_millis(frame.delay_ms as u64);
            Some(if let Ok(duration) = ts_previous_cb.elapsed() {
                // dbg!(interval, duration);
                if interval > duration {
                    interval - duration
                } else {
                    Duration::from_millis(1)
                }
            } else {
                interval
            })
        } else {
            None
        }
    }

    fn advance(&mut self, _current_time: SystemTime) -> Option<Pixbuf> {
        self.index += 1;
        if self.index >= self.decoder.num_frames() {
            self.index = 0;
            self.first_run = false;
        }
        if self.first_run {
            if let Ok((pixbuf, delay_ms)) = WebP::read_frame(&mut self.decoder) {
                self.frames.push(AnimationFrame {
                    delay_ms,
                    pixbuf: pixbuf.clone(),
                });
                Some(pixbuf)
            } else {
                None
            }
        } else {
            self.pixbuf_get(self.index as usize)
        }
    }

    pub fn pixbuf_get(&self, index: usize) -> Option<Pixbuf> {
        self.frames.get(index).map(|frame| frame.pixbuf.clone())
    }
}
