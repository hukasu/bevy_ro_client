use std::io::Read;

use crate::reader_ext::ReaderExt;

use super::Version;

#[derive(Debug)]
pub struct Light {
    pub name: Box<str>,
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub range: f32,
}

impl Light {
    pub fn from_reader(
        mut reader: &mut dyn Read,
        _version: &Version,
    ) -> Result<Self, std::io::Error> {
        let name = crate::assets::read_euc_kr_string(reader, 80)?;

        let position = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];

        let color = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let range = reader.read_le_f32()?;

        Ok(Self {
            name,
            position,
            color,
            range,
        })
    }
}
