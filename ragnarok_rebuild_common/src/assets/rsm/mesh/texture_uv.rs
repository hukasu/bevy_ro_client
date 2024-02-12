use crate::{assets::common::Version, reader_ext::ReaderExt};
use std::io::{self, Read};

#[derive(Debug)]
pub struct TextureUV {
    pub color: [u8; 4],
    pub uv: [f32; 2],
}

impl TextureUV {
    pub fn from_reader(mut reader: &mut dyn Read, version: &Version) -> Result<Self, io::Error> {
        Ok(Self {
            color: if version >= &Version(1, 2, 0) {
                [
                    reader.read_u8()?,
                    reader.read_u8()?,
                    reader.read_u8()?,
                    reader.read_u8()?,
                ]
            } else {
                [0xff, 0xff, 0xff, 0xff]
            },
            uv: [reader.read_le_f32()?, reader.read_le_f32()?],
        })
    }
}
