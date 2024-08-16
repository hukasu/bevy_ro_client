use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::read_euc_kr_string;

use super::Version;

#[derive(Debug)]
pub struct Model {
    pub name: Box<str>,
    pub animation_type: i32,
    pub animation_speed: f32,
    pub block_type: i32,
    pub filename: Box<str>,
    pub node_name: Box<str>,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Model {
    pub fn from_reader(
        mut reader: &mut dyn Read,
        _version: &Version,
    ) -> Result<Self, std::io::Error> {
        let name = read_euc_kr_string(reader, 40)?;
        let animation_type = reader.read_le_i32()?;
        let animation_speed = reader.read_le_f32()?;
        let block_type = reader.read_le_i32()?;
        let filename = read_euc_kr_string(reader, 80)?;
        // There are models were this field is corrupt
        let node_name = read_euc_kr_string(reader, 80).unwrap_or_default();

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
        let scale = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];

        Ok(Self {
            name,
            animation_type,
            animation_speed,
            block_type,
            filename,
            node_name,
            position,
            rotation,
            scale,
        })
    }
}