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
    env,
    fs::{self, File},
    io::{self, BufWriter, Write},
    sync::OnceLock,
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
}

impl Config {
    pub fn home() -> String {
        env::var("HOME").unwrap_or_default()
    }

    fn dir() -> String {
        format!("{}/.config/mview", Self::home())
    }

    fn filename() -> String {
        format!("{}/mview6.json", Self::dir())
    }

    pub fn save(&self) -> std::io::Result<()> {
        fs::create_dir_all(Self::dir())?;
        let file = File::create(Self::filename())?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, self)?;
        writer.flush()?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let config = Self {
            bookmarks: vec![Bookmark {
                name: "Home folder".to_string(),
                folder: Self::home(),
            }],
        };
        match config.save() {
            Ok(_) => println!("Saved default configuration to {}", Self::filename()),
            Err(_) => println!(
                "Failed to save default configuration to {}",
                Self::filename()
            ),
        };
        config
    }
}

fn read_config() -> io::Result<Config> {
    let file = fs::File::open(Config::filename())?;
    let config: Config = serde_json::from_reader(file)?;
    println!("deserialized = {:?}", config);
    Ok(config)
}

pub fn config<'a>() -> &'a Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| read_config().unwrap_or_default())
}
