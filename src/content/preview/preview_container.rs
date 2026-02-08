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
    fs::File,
    io::{BufRead, BufReader, ErrorKind, Result, Write},
    path::Path,
};

use crate::{content::PreviewImage, error::MviewResult};

pub struct PreviewContainer {
    images: Vec<PreviewImage>,
}

impl PreviewContainer {
    pub fn new(images: Vec<PreviewImage>) -> Self {
        Self { images }
    }

    // 4 MPRE
    // 4 version
    // 4 number of images

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(b"MPRE")?;
        writer.write_all(&(1_u32).to_le_bytes())?; // 4
        writer.write_all(&(self.images.len() as u32).to_le_bytes())?; // 4
        for image in &self.images {
            image.write(writer)?;
        }
        Ok(())
    }

    pub fn read<T: BufRead>(reader: &mut T) -> Result<Self> {
        let mut buf = [0u8; 12];
        reader.read_exact(&mut buf)?;

        let version = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let number_of_images = u32::from_le_bytes(buf[8..12].try_into().unwrap());

        if &buf[0..4] != b"MPRE" || version != 1 {
            return Err(ErrorKind::InvalidData.into());
        }

        // Sanity check on values
        if number_of_images > 128 {
            return Err(ErrorKind::FileTooLarge.into());
        }

        let mut images = Vec::new();

        for _ in 0..number_of_images {
            images.push(PreviewImage::read(reader)?);
        }

        Ok(Self { images })
    }

    pub fn load(path: &Path) -> MviewResult<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let containter = Self::read(&mut reader)?;
        Ok(containter)
    }

    pub fn save(&self, path: &Path) -> MviewResult<()> {
        let mut file = File::create(path)?;
        self.write(&mut file)?;
        Ok(())
    }

    pub fn image(&self, index: usize) -> Option<&PreviewImage> {
        self.images.get(index)
    }
}
