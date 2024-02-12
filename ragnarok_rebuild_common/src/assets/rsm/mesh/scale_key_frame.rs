use std::io::{self, Read};

use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct ScaleKeyFrame {
    pub frame: i32,
    pub scale: [f32; 3],
    pub data: f32, // Unknown purpose
}

impl ScaleKeyFrame {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, io::Error> {
        let frame = reader.read_le_i32()?;
        let scale = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let data = reader.read_le_f32()?;

        Ok(Self { frame, scale, data })
    }
}
