use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct BoundingBox {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

impl BoundingBox {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, std::io::Error> {
        let top = reader.read_le_i32()?;
        let bottom = reader.read_le_i32()?;
        let left = reader.read_le_i32()?;
        let right = reader.read_le_i32()?;

        Ok(Self {
            top,
            bottom,
            left,
            right,
        })
    }
}
