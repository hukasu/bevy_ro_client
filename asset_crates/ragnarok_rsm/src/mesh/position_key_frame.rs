use std::io::{self, Read};

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct PositionKeyFrame {
    pub frame: i32,
    pub position: [f32; 3],
    pub data: f32, // Unknown purpose
}

impl PositionKeyFrame {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, io::Error> {
        let frame = reader.read_le_i32()?;
        let position = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let data = reader.read_le_f32()?;

        Ok(Self {
            frame,
            position,
            data,
        })
    }
}
