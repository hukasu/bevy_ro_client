use std::io::Read;

use crate::reader_ext::ReaderExt;

use super::Version;

#[derive(Debug)]
pub struct Effect {
    pub name: Box<str>,
    pub position: (f32, f32, f32),
    pub id: i32,
    pub delay: f32,
    pub parameters: (f32, f32, f32, f32),
}

impl Effect {
    pub fn from_reader(
        mut reader: &mut dyn Read,
        _version: &Version,
    ) -> Result<Self, std::io::Error> {
        let name = crate::assets::read_euc_kr_string(reader, 80)?;

        // Ragnarok seems to be Y-up left-handed coordinate system with Z backwards
        // Bevy is Y-up right-handed coordinate system with Z forwards
        // https://bevy-cheatbook.github.io/img/handedness.png
        let position = (
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            -reader.read_le_f32()?,
        );

        let id = reader.read_le_i32()?;
        let delay = reader.read_le_f32()?;
        let parameters = (
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        );

        Ok(Self {
            name,
            position,
            id,
            delay,
            parameters,
        })
    }
}
