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

use crate::{
    backends::{Backend, ImageParams},
    classification::{FileClassification, FileType, Preference},
    content::{Content, ContentLoader},
    error::MviewResult,
    file_view::{
        Direction, Target, {BackendRef, FileRow, FileStore, ItemRef, Reference},
    },
    image::{InternalImageLoader, RsImageLoader},
    mview6_error,
    util::path_to_filename,
};
use image::DynamicImage;
use regex::Regex;
use std::{
    fs::{metadata, read_dir, rename},
    io::{self},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::OnceLock,
    time::UNIX_EPOCH,
};

fn extension_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\.([^.]+)$").expect("valid regex"))
}

/// Remove a `.marker` segment from the stem of a filename, leaving the extension intact.
/// E.g. `photo.lo.jpg` with marker `"lo"` → `photo.jpg`.
fn remove_marker(filename: &str, marker: &str) -> String {
    let marker_suffix = format!(".{marker}");
    if let Some(dot_pos) = filename.rfind('.') {
        let stem = &filename[..dot_pos];
        let ext = &filename[dot_pos..];
        if let Some(new_stem) = stem.strip_suffix(&marker_suffix) {
            return format!("{new_stem}{ext}");
        }
    }
    filename.to_string()
}

pub struct FileSystem {
    directory: PathBuf,
    store: FileStore,
}

impl FileSystem {
    pub fn try_new(directory: &Path) -> MviewResult<Self> {
        Ok(Self {
            directory: directory.into(),
            store: Self::read_directory(directory)?,
        })
    }

    fn read_directory(current_dir: &Path) -> io::Result<FileStore> {
        let store = FileRow::empty_store();
        for entry in read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            let filename = path_to_filename(&path);

            if filename.starts_with('.') {
                continue;
            }

            let metadata = match metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    println!("{filename}: Err = {e:?}");
                    continue;
                }
            };

            let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
            let modified = if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                duration.as_secs()
            } else {
                0
            };
            let size = metadata.len();

            let classification = FileClassification::determine(&path, metadata.is_dir());

            store.append(&FileRow::new(
                classification,
                filename.to_string(),
                size,
                modified,
            ));
        }
        Ok(store)
    }

    pub fn get_thumbnail(src: &Reference) -> MviewResult<DynamicImage> {
        if let (BackendRef::FileSystem(directory), ItemRef::String(name)) = src.as_tuple() {
            let filename = directory.join(name);
            if let Some(image) = InternalImageLoader::thumb_from_file(&filename) {
                Ok(image)
            } else {
                let thumb_filename = name.replace(".lo.", ".").replace(".hi.", ".") + ".mthumb";
                let thumb_path = directory.join(".mview").join(thumb_filename);
                if Path::new(&thumb_path).exists() {
                    RsImageLoader::dynimg_from_file(&thumb_path)
                } else {
                    let path = directory.join(name);
                    let image = RsImageLoader::dynimg_from_file(&path)?;
                    let image = image.resize(175, 175, image::imageops::FilterType::Lanczos3);
                    // ImageSaver::save_thumbnail(&src.directory, &thumb_filename, &image);
                    Ok(image)
                }
            }
        } else {
            mview6_error!("invalid reference").into()
        }
    }
}

impl Backend for FileSystem {
    fn class_name(&self) -> &str {
        "FileSystem"
    }

    fn path(&self) -> PathBuf {
        self.directory.clone()
    }

    fn list(&self) -> FileStore {
        self.store.clone()
    }

    fn enter(&self, row: &FileRow) -> Option<Box<dyn Backend>> {
        let file_type = row.file_type();
        if file_type == FileType::Video {
            let full_path = self.directory.join(row.name());
            println!("Launch video external {}", full_path.to_string_lossy());
            let child = Command::new("mpv")
                .arg(full_path)
                .arg("--fullscreen=yes")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            if let Err(error) = child {
                eprintln!("Failed to launch mpv {:?}", error);
            };
            None
        } else if file_type == FileType::Folder
            || file_type == FileType::Archive
            || file_type == FileType::Document
        {
            <dyn Backend>::new_from_path(&self.directory.join(row.name())).ok()
        } else {
            None
        }
    }

    fn leave(&self) -> Option<(Box<dyn Backend>, Target)> {
        if let Some(parent) = self.directory.parent() {
            match Self::try_new(parent) {
                Ok(new_backend) => Some((
                    Box::new(new_backend),
                    Target::Name(path_to_filename(&self.directory)),
                )),
                Err(e) => {
                    eprintln!("Failed to leave directory: {e}");
                    None
                }
            }
        } else {
            None
        }
    }

    fn content(&self, item: &ItemRef, _: &ImageParams) -> Content {
        let path = self.directory.join(item.str());
        let mut content = ContentLoader::content_from_file(&path);
        content.path = Some(path);
        content
    }

    fn set_preference(&self, row: &FileRow, direction: Direction) -> bool {
        let file_type = row.file_type();
        if file_type != FileType::Image {
            //TODO: drop this restriction?
            return false;
        }

        let filename = row.name();
        let (new_filename, new_preference) = if matches!(direction, Direction::Up) {
            if filename.contains(".hi.") {
                return true;
            } else if filename.contains(".lo.") {
                (remove_marker(&filename, "lo"), Preference::Normal)
            } else {
                (
                    extension_re().replace(&filename, ".hi.$1").to_string(),
                    Preference::Liked,
                )
            }
        } else if filename.contains(".lo.") {
            return true;
        } else if filename.contains(".hi.") {
            (remove_marker(&filename, "hi"), Preference::Normal)
        } else {
            (
                extension_re().replace(&filename, ".lo.$1").to_string(),
                Preference::Disliked,
            )
        };
        dbg!(&self.directory, &filename, &new_filename);
        match rename(
            self.directory.join(&filename),
            self.directory.join(&new_filename),
        ) {
            Ok(()) => {
                row.set_name(new_filename);
                row.set_preference(new_preference);
                true
            }
            Err(e) => {
                println!("Failed to rename {filename} to {new_filename}: {e:?}");
                false
            }
        }
    }

    fn backend_ref(&self) -> BackendRef {
        BackendRef::FileSystem(self.directory.clone())
    }

    fn reload(&self) -> Option<Box<dyn Backend>> {
        let directory = &self.directory;
        Some(Box::new(FileSystem {
            directory: directory.into(),
            store: Self::read_directory(directory).unwrap_or_else(|_| FileRow::empty_store()),
        }))
    }
}

// fn _read_bytes(path: &Path) -> MviewResult<Vec<u8>> {
//     let file = File::open(path)?;
//     let mut buffer = Vec::new();
//     file.take(_MAX_CONTENT_SIZE).read_to_end(&mut buffer)?;
//     Ok(buffer)
// }
