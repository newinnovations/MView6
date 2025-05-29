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

use std::{panic, thread, time};

use async_channel::Sender;
use image::DynamicImage;

use crate::{
    backends::{
        archive_mar::MarArchive, archive_rar::RarArchive, archive_zip::ZipArchive,
        document::Document, filesystem::FileSystem,
    },
    category::Category,
    error::MviewResult,
    image::{draw::text_thumb, provider::image_rs::RsImageLoader, view::ImageView},
};

use super::{Message, TCommand, TMessage, TReference, TResult, TResultOption, TTask};

fn thumb_result(res: MviewResult<DynamicImage>, task: &TTask) -> TResultOption {
    match res {
        Ok(image) => {
            let image = image.resize(task.size, task.size, image::imageops::FilterType::Lanczos3);
            TResultOption::Image(image)
        }
        Err(_error) => match task.source.category {
            Category::Folder => TResultOption::Message(TMessage::new(
                &task.source.category.name(),
                &task.source.name,
                task.source.category.colors(),
            )),
            Category::Archive => TResultOption::Message(TMessage::new(
                &task.source.category.name(),
                &task.source.name,
                task.source.category.colors(),
            )),
            Category::Unsupported => TResultOption::Message(TMessage::new(
                &task.source.category.name(),
                &task.source.name,
                task.source.category.colors(),
            )),
            _ => TResultOption::Message(TMessage::error("error", &task.source.name)),
        },
    }
}

pub fn start_thumbnail_task(
    sender: &Sender<Message>,
    image_view: &ImageView,
    command: &TCommand,
    current_task: &mut usize,
) {
    // let elapsed = command.elapsed();
    // println!("ThumbnailTask: {:7.3}", elapsed);
    let id = image_view.image_id();
    if command.id == id {
        // println!("-- command id is ok: {id}");
        let sender_clone = sender.clone();
        if let Some(task) = command.tasks.get(*current_task) {
            *current_task += 1;
            let task = task.clone();
            // let tid = task.tid;
            thread::spawn(move || {
                // println!("{tid:3}: start {:7.3}", elapsed);
                // thread::sleep(time::Duration::from_secs(2));
                thread::sleep(time::Duration::from_millis(1));
                let result = match panic::catch_unwind(|| match &task.source.reference {
                    TReference::FileReference(src) => {
                        thumb_result(FileSystem::get_thumbnail(src), &task)
                    }
                    TReference::ZipReference(src) => {
                        thumb_result(ZipArchive::get_thumbnail(src), &task)
                    }
                    TReference::MarReference(src) => {
                        thumb_result(MarArchive::get_thumbnail(src), &task)
                    }
                    TReference::RarReference(src) => {
                        thumb_result(RarArchive::get_thumbnail(src), &task)
                    }
                    TReference::DocReference(src) => {
                        thumb_result(Document::get_thumbnail(src), &task)
                    }
                    TReference::None => {
                        TResultOption::Message(TMessage::error("none", "TEntry::None"))
                    }
                }) {
                    Ok(image) => image,
                    Err(_) => TResultOption::Message(TMessage::error("panic", &task.source.name)),
                };
                let _ = sender_clone.send_blocking(Message::Result(TResult::new(id, task, result)));
            });
        }
    } else {
        // println!("-- command id mismatch {} != {id}", command.id);
    }
}

pub fn handle_thumbnail_result(
    image_view: &ImageView,
    command: &mut TCommand,
    result: TResult,
) -> bool {
    if command.id != result.id {
        return false;
    }
    // let tid = result.task.tid;
    let elapsed = command.elapsed();
    command.todo -= 1;
    // println!("{tid:3}: ready {:7.3} todo={}", elapsed, command.todo);
    if result.id == image_view.image_id() {
        // println!("{tid:3}: -- result id is ok: {id}");

        let pixbuf = match result.result {
            TResultOption::Image(image) => RsImageLoader::dynimg_to_pixbuf(image),
            TResultOption::Message(message) => text_thumb(message),
        };

        match pixbuf {
            Ok(thumb_pb) => {
                let size = result.task.size as i32;

                let thumb_pb = if thumb_pb.width() > size || thumb_pb.height() > size {
                    RsImageLoader::pixbuf_scale(thumb_pb, size)
                } else {
                    Some(thumb_pb)
                };

                if let Some(thumb_pb) = thumb_pb {
                    let (x, y) = result.task.position;
                    image_view.draw_pixbuf(
                        &thumb_pb,
                        x + (size - thumb_pb.width()) / 2,
                        y + (size - thumb_pb.height()) / 2,
                    );
                }
            }
            Err(error) => {
                println!("Thumbnail: failed to convert to pixbuf {:?}", error);
            }
        }
        if command.todo == 0 || (elapsed - command.last_update) > 0.3 {
            if command.last_update == 0.0 {
                image_view.set_image_post();
            }
            image_view.image_modified();
            command.last_update = elapsed;
        }
        return command.todo != 0;
    } else {
        // println!("{tid:3}: -- command id mismatch {} != {id}", result.id);
    }
    false
}
