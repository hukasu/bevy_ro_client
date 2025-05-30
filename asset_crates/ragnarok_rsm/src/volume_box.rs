use std::io::{self, Read};

#[cfg(feature = "bevy")]
use bevy_reflect::Reflect;

use ragnarok_rebuild_common::{Version, reader_ext::ReaderExt};

#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
pub struct VolumeBox {
    pub size: [f32; 3],
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub flag: i32,
}

impl VolumeBox {
    pub fn from_reader(mut reader: &mut dyn Read, version: &Version) -> Result<Self, io::Error> {
        let size = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let position = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let rotation = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let flag = if version >= &Version(1, 3, 0) {
            reader.read_le_i32()?
        } else {
            0
        };
        Ok(Self {
            size,
            position,
            rotation,
            flag,
        })
    }
}
