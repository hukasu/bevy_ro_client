use std::io::{self, Read};

use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct TextureAnimation {
    pub texture_id: i32,
    pub animations: Box<[Animation]>,
}

impl TextureAnimation {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, io::Error> {
        let texture_id = reader.read_le_i32()?;
        let animation_count = reader.read_le_i32()?;

        let animations = (0..animation_count)
            .map(|_| Animation::from_reader(reader))
            .collect::<Result<Box<[Animation]>, io::Error>>()?;

        Ok(TextureAnimation {
            texture_id,
            animations,
        })
    }
}

#[derive(Debug)]
pub struct Animation {
    pub animation_type: i32,
    pub key_frames: Box<[(i32, f32)]>,
}

impl Animation {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, io::Error> {
        let animation_type = reader.read_le_i32()?;
        let animation_frames = reader.read_le_i32()?;
        let key_frames = (0..animation_frames)
            .map(|_| {
                let frame = reader.read_le_i32()?;
                let offset = reader.read_le_f32()?;
                Ok((frame, offset))
            })
            .collect::<Result<Box<[(i32, f32)]>, io::Error>>()?;
        Ok(Animation {
            animation_type,
            key_frames,
        })
    }
}
