use std::io::Read;

use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct LightingParams {
    pub longitude: u32,
    pub latitude: u32,
    pub diffuse_red: f32,
    pub diffuse_green: f32,
    pub diffuse_blue: f32,
    pub ambient_red: f32,
    pub ambient_green: f32,
    pub ambient_blue: f32,
    pub shadow_map_alpha: f32,
}

impl LightingParams {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, std::io::Error> {
        let longitude = reader.read_le_u32()?;
        let latitude = reader.read_le_u32()?;
        let diffuse_red = reader.read_le_f32()?;
        let diffuse_green = reader.read_le_f32()?;
        let diffuse_blue = reader.read_le_f32()?;
        let ambient_red = reader.read_le_f32()?;
        let ambient_green = reader.read_le_f32()?;
        let ambient_blue = reader.read_le_f32()?;
        let shadow_map_alpha = reader.read_le_f32()?;

        Ok(Self {
            longitude,
            latitude,
            diffuse_red,
            diffuse_green,
            diffuse_blue,
            ambient_red,
            ambient_green,
            ambient_blue,
            shadow_map_alpha,
        })
    }
}
