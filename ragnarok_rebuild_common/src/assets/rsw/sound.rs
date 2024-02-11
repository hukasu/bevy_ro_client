use std::io::Read;

use crate::reader_ext::ReaderExt;

use super::Version;

#[derive(Debug)]
pub struct Sound {
    pub name: Box<str>,
    pub filename: Box<str>,
    pub position: (f32, f32, f32),
    pub volume: f32,
    pub width: i32,
    pub height: i32,
    pub range: f32,
    pub cycle: f32,
}

impl Sound {
    pub fn from_reader(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Self, std::io::Error> {
        let name = crate::assets::read_euc_kr_string(reader, 80)?;
        let filename = crate::assets::read_euc_kr_string(reader, 80)?;

        // Ragnarok seems to be Y-up left-handed coordinate system with Z backwards
        // Bevy is Y-up right-handed coordinate system with Z forwards
        // https://bevy-cheatbook.github.io/img/handedness.png
        let position = (
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            -reader.read_le_f32()?,
        );

        let volume = reader.read_le_f32()?;
        let width = reader.read_le_i32()?;
        let height = reader.read_le_i32()?;
        let range = reader.read_le_f32()?;
        let cycle = if version >= &Version(2, 1, 0) {
            reader.read_le_f32()?
        } else {
            0.
        };

        Ok(Self {
            name,
            filename,
            position,
            volume,
            width,
            height,
            range,
            cycle,
        })
    }
}
