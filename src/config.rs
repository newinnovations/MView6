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
    fs::{create_dir_all, File},
    io::{BufWriter, Result, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicI32, Ordering},
        OnceLock,
    },
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Bookmark {
    pub name: String,
    pub folder: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub bookmarks: Vec<Bookmark>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast: Option<i32>,
}

fn pathbuf_to_string(pathbuf: &Path) -> String {
    pathbuf.to_str().unwrap_or_default().to_string()
}

impl Config {
    fn config_dir() -> PathBuf {
        let mut dir = dirs::config_dir().unwrap_or_default();
        dir.push("mview6");
        dbg!(&dir);
        dir
    }

    fn config_file() -> PathBuf {
        Self::config_dir().join("mview6.json")
    }

    pub fn save(&self) -> std::io::Result<()> {
        create_dir_all(Self::config_dir())?;
        let file = File::create(Self::config_file())?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
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

        let config = Self {
            bookmarks,
            contrast: None,
        };

        match config.save() {
            Ok(_) => println!("Saved default configuration to {:?}", Self::config_file()),
            Err(_) => println!(
                "Failed to save default configuration to {:?}",
                Self::config_file()
            ),
        };
        config
    }
}

fn read_config() -> Result<Config> {
    let file = File::open(Config::config_file())?;
    let config: Config = serde_json::from_reader(file)?;
    println!("deserialized = {:?}", config);
    Ok(config)
}

pub fn config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| read_config().unwrap_or_default())
}

static CONTRAST: AtomicI32 = AtomicI32::new(0);

pub fn contrast_delta(delta: i32) {
    CONTRAST.store(CONTRAST.load(Ordering::Relaxed) + delta, Ordering::Relaxed);
}

pub fn contrast() -> u8 {
    let mut contrast = CONTRAST.load(Ordering::Relaxed);
    if let Some(initial) = config().contrast {
        contrast += initial;
    }
    contrast as u8
}
