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

use std::fs::File;
use std::io::{self, Read};

#[allow(dead_code)]
#[derive(Debug)]
pub struct MemoryUsage {
    total_program_size: usize,
    resident_set_size: usize,
    shared_pages: usize,
    text: usize,
    library: usize,
    data: usize,
    dt: usize,
}

fn read_memory_usage() -> io::Result<String> {
    let mut file = File::open("/proc/self/statm")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn get_memory_usage() -> Result<MemoryUsage, &'static str> {
    if let Ok(data) = read_memory_usage() {
        let fields: Vec<&str> = data.split_whitespace().collect();
        if fields.len() != 7 {
            return Err("Unexpected number of fields in /proc/self/statm");
        }

        let total_program_size = fields[0]
            .parse::<usize>()
            .map_err(|_| "Failed to parse total program size")?;
        let resident_set_size = fields[1]
            .parse::<usize>()
            .map_err(|_| "Failed to parse resident set size")?;
        let shared_pages = fields[2]
            .parse::<usize>()
            .map_err(|_| "Failed to parse shared pages")?;
        let text = fields[3]
            .parse::<usize>()
            .map_err(|_| "Failed to parse text")?;
        let library = fields[4]
            .parse::<usize>()
            .map_err(|_| "Failed to parse library")?;
        let data = fields[5]
            .parse::<usize>()
            .map_err(|_| "Failed to parse data")?;
        let dt = fields[6]
            .parse::<usize>()
            .map_err(|_| "Failed to parse dt")?;

        Ok(MemoryUsage {
            total_program_size,
            resident_set_size,
            shared_pages,
            text,
            library,
            data,
            dt,
        })
    } else {
        Err("Failed to read /proc/self/statm")
    }
}

#[allow(dead_code)]
pub fn dump_memory_usage() {
    match get_memory_usage() {
        Ok(usage) => {
            dbg!(usage);
        }
        Err(error) => {
            println!("{error}");
        }
    }
}

pub fn memory_short() -> String {
    match get_memory_usage() {
        Ok(usage) => {
            format!("(rss={0}, data={1})", usage.resident_set_size, usage.data)
        }
        Err(_) => String::default(),
    }
}
