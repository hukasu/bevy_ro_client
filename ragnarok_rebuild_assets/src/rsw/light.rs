use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct Light {
    pub name: Box<str>,
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub range: f32,
}

impl Light {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, std::io::Error> {
        let name = crate::read_euc_kr_string(reader, 80)?;

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
