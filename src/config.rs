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
    fs::{create_dir_all, File},
    io::{BufWriter, Result, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicI32, Ordering},
        OnceLock,
    },
};

use std::io;

use serde::{Deserialize, Serialize};
use syntect::{highlighting::ThemeSet, parsing::SyntaxSet};

#[derive(Serialize, Deserialize, Debug)]
pub struct Bookmark {
    pub name: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub bookmarks: Vec<Bookmark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast: Option<i32>,
}

#[derive(Debug)]
pub struct Config {
    pub config_file: ConfigFile,
    pub ps: SyntaxSet,
    pub ts: ThemeSet,
}

fn pathbuf_to_string(pathbuf: &Path) -> String {
    pathbuf.to_string_lossy().into_owned()
}

impl ConfigFile {
    fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|mut dir| {
            dir.push("mview6");
            dir
        })
    }

    pub fn config_file() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("mview6.json"))
    }

    pub fn save(&self) -> std::io::Result<()> {
        let dir = Self::config_dir().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "config directory unavailable")
        })?;
        create_dir_all(&dir)?;
        let path = dir.join("mview6.json");
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;
        Ok(())
    }

    fn new_default() -> Self {
        let mut bookmarks = Vec::<Bookmark>::new();

        if let Some(dir) = dirs::home_dir() {
            bookmarks.push(Bookmark {
                name: "Home folder".to_string(),
                folder: pathbuf_to_string(&dir),
            });
        }

        if let Some(dir) = dirs::picture_dir() {
            bookmarks.push(Bookmark {
                name: "Pictures folder".to_string(),
                folder: pathbuf_to_string(&dir),
            });
        }

        if let Some(dir) = dirs::document_dir() {
            bookmarks.push(Bookmark {
                name: "Document folder".to_string(),
                folder: pathbuf_to_string(&dir),
            });
        }

        if let Some(dir) = dirs::download_dir() {
            bookmarks.push(Bookmark {
                name: "Download folder".to_string(),
                folder: pathbuf_to_string(&dir),
            });
        }

        Self {
            bookmarks,
            contrast: None,
        }
    }
}

/// `Default` is a pure in-memory construction with no I/O side-effects.
impl Default for ConfigFile {
    fn default() -> Self {
        Self::new_default()
    }
}

fn read_config() -> Result<ConfigFile> {
    let path = ConfigFile::config_file()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "config directory unavailable"))?;
    let file = File::open(&path)?;
    let config: ConfigFile = serde_json::from_reader(file)?;
    Ok(config)
}

pub fn config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let config_file = match ConfigFile::config_file() {
            None => {
                eprintln!("Config directory unavailable; using in-memory defaults");
                ConfigFile::default()
            }
            Some(path) => {
                if path.exists() {
                    match read_config() {
                        Ok(cfg) => {
                            println!("Config file location {:?}", path);
                            cfg
                        }
                        Err(e) => {
                            // Preserve the original file — do not overwrite a malformed config.
                            eprintln!(
                                "Failed to parse config {:?}: {e}; using in-memory defaults (original file preserved)",
                                path
                            );
                            ConfigFile::default()
                        }
                    }
                } else {
                    // File does not exist yet — create it with defaults.
                    let cfg = ConfigFile::default();
                    match cfg.save() {
                        Ok(_) => println!("Saved default configuration to {:?}", path),
                        Err(e) => eprintln!(
                            "Failed to save default configuration to {:?}: {e}",
                            path
                        ),
                    }
                    cfg
                }
            }
        };
        Config {
            config_file,
            ps: SyntaxSet::load_defaults_nonewlines(),
            ts: ThemeSet::load_defaults(),
        }
    })
}

static CONTRAST: AtomicI32 = AtomicI32::new(0);

pub fn contrast_delta(delta: i32) {
    CONTRAST.store(CONTRAST.load(Ordering::Relaxed) + delta, Ordering::Relaxed);
}

pub fn contrast() -> u8 {
    let mut contrast = CONTRAST.load(Ordering::Relaxed);
    if let Some(initial) = config().config_file.contrast {
        contrast += initial;
    }
    contrast as u8
}
