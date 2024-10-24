use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;
#[derive(Debug)]
pub struct Frame {
    pub index: u32,
    pub origin_u: i32,
    pub origin_v: i32,
}

impl Frame {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let index = reader.read_le_u32()?;
        let origin_u = reader.read_le_i32()?;
        let origin_v = reader.read_le_i32()?;

        Ok(Self {
            index,
            origin_u,
            origin_v,
        })
    }
}
