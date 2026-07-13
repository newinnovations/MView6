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

use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::{
    content::{PreviewCaption, PreviewContainer, PreviewImage},
    error::MviewResult,
    mview6_error,
    profile::performance::Performance,
};

const TOTAL_FRAMES: u32 = 16;

pub struct VideoPreview {}

impl VideoPreview {
    /// `progress` is called after every frame has been extracted with
    /// `(frames_done, total_frames)`.
    pub fn create(path: &Path, progress: &dyn Fn(u32, u32)) -> MviewResult<PreviewContainer> {
        let video_path = path.to_string_lossy().to_string();

        // Get video duration
        let total_duration = Self::get_video_duration(&video_path)?;
        println!("Video duration: {:.2} seconds", total_duration);

        let video_len_cs = (total_duration * 100.0).round() as u32;
        let interval = total_duration / (TOTAL_FRAMES + 1) as f64;

        // Generate thumbnail images
        let mut images = Vec::new();
        for i in 1..=TOTAL_FRAMES {
            let duration = Performance::start();
            let timestamp = interval * i as f64;
            // Extract frame directly to memory
            let img_data = Self::extract_frame_to_memory(&video_path, timestamp)?;
            // Load image from memory
            let image = PreviewImage::new(
                image::load_from_memory(&img_data)?,
                PreviewCaption::Video {
                    video_length: video_len_cs,
                    captured_at: (timestamp * 100.0).round() as u32,
                },
            )?;
            images.push(image);
            duration.elapsed(&format!("preview {i} of {TOTAL_FRAMES}"));
            progress(i, TOTAL_FRAMES);
        }

        Ok(PreviewContainer::new(images))
    }

    fn get_video_duration(video_path: &str) -> MviewResult<f64> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                video_path,
            ])
            .output();

        let output = match output {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute ffprobe: {}", e);
                return mview6_error!(
                    "Failed to execute ffprobe\n\nIs it installed and in your PATH?"
                )
                .into();
            }
        };

        let duration_str = String::from_utf8_lossy(&output.stdout);
        Ok(duration_str.trim().parse().unwrap_or(0.0))
    }

    fn extract_frame_to_memory(video_path: &str, timestamp: f64) -> MviewResult<Vec<u8>> {
        let output = Command::new("ffmpeg")
            .args([
                "-ss",
                &timestamp.to_string(),
                "-i",
                video_path,
                "-vframes",
                "1",
                "-q:v",
                "2",
                "-f",
                "image2pipe",
                "-vcodec",
                "mjpeg",
                "-",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output();

        let output = match output {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to execute ffmpeg: {}", e);
                return mview6_error!(
                    "Failed to execute ffmpeg\n\nIs it installed and in your PATH?"
                )
                .into();
            }
        };

        Ok(output.stdout)
    }
}
