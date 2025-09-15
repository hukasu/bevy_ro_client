use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct LightingParams {
    pub longitude: u32,
    pub latitude: u32,
    pub diffuse_color: [f32; 3],
    pub ambient_color: [f32; 3],
    pub shadow_map_alpha: f32,
}

impl LightingParams {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, std::io::Error> {
        let longitude = reader.read_le_u32()?;
        let latitude = reader.read_le_u32()?;
        let diffuse = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let ambient = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];
        let shadow_map_alpha = reader.read_le_f32()?;

        Ok(Self {
            longitude,
            latitude,
            diffuse_color: diffuse,
            ambient_color: ambient,
            shadow_map_alpha,
        })
    }
}
