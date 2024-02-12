use std::io::{self, Read};

use crate::{assets::common::Version, reader_ext::ReaderExt};

#[derive(Debug)]
pub struct Face {
    pub vertices: [u16; 3],
    pub uv: [u16; 3],
    pub texture_id: u16,
    pub two_side: u8,
    pub smoothing_group: Box<[i32]>,
}

impl Face {
    pub fn from_reader(mut reader: &mut dyn Read, version: &Version) -> Result<Self, io::Error> {
        let len = if version >= &Version(2, 2, 0) {
            reader.read_le_i32()?
        } else {
            24
        };

        let vertices = [
            reader.read_le_u16()?,
            reader.read_le_u16()?,
            reader.read_le_u16()?,
        ];
        let uv = [
            reader.read_le_u16()?,
            reader.read_le_u16()?,
            reader.read_le_u16()?,
        ];
        let texture_id = reader.read_le_u16()?;

        let flags = reader.read_vec(6)?;
        let two_side = flags[2];

        let smoothing_group = if version >= &Version(1, 2, 0) {
            let count = (len - 20) / 4;
            (0..count)
                .map(|_| reader.read_le_i32())
                .collect::<Result<Box<[i32]>, io::Error>>()?
        } else {
            [].into()
        };

        Ok(Self {
            vertices,
            uv,
            texture_id,
            two_side,
            smoothing_group,
        })
    }
}
