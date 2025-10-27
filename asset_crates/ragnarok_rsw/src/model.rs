use std::io::Read;

use ragnarok_rebuild_common::{Version, euc_kr::read_euc_kr_string, reader_ext::ReaderExt};

#[derive(Debug, Clone)]
pub struct Model {
    pub name: Box<str>,
    pub animation_type: i32,
    pub animation_speed: f32,
    pub block_type: i32,
    pub flag: u8,
    pub extra_flag: u32,
    pub filename: Box<str>,
    pub node_name: Box<str>,
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Model {
    pub fn from_reader(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Self, std::io::Error> {
        let name = read_euc_kr_string(reader, 40)?;
        let animation_type = reader.read_le_i32()?;
        let animation_speed = reader.read_le_f32()?;
        let block_type = reader.read_le_i32()?;
        let flag = match version {
            Version(2, 6, 187) | Version(2, 6, 197) | Version(2, 7, _) => reader.read_u8()?,
            _ => 0,
        };
        let extra_flag = match version {
            Version(2, 7, _) => reader.read_le_u32()?,
            _ => 0,
        };
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
            flag,
            extra_flag,
            filename,
            node_name,
            position,
            rotation,
            scale,
        })
    }
}
