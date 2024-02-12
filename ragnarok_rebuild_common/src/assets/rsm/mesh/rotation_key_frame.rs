use std::io::{self, Read};

use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct RotationKeyFrame {
    pub frame: i32,
    pub quaternion: [f32; 4],
}

impl RotationKeyFrame {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, io::Error> {
        let frame = reader.read_le_i32()?;
        let quaternion = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];

        Ok(Self { frame, quaternion })
    }
}
