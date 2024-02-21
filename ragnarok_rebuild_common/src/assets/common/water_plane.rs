use std::io::Read;

use crate::reader_ext::ReaderExt;

#[derive(Debug, Clone)]
pub struct WaterPlane {
    pub water_level: f32,
    pub water_type: i32,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_pitch: f32,
    pub texture_cyclical_interval: i32,
}

impl WaterPlane {
    pub fn read_single(mut reader: &mut dyn Read) -> Result<Self, std::io::Error> {
        let water_level = reader.read_le_f32()?;
        let water_type = reader.read_le_i32()?;
        let wave_height = reader.read_le_f32()?;
        let wave_speed = reader.read_le_f32()?;
        let wave_pitch = reader.read_le_f32()?;
        let texture_cyclical_interval = reader.read_le_i32()?;

        Ok(Self {
            water_level,
            water_type,
            wave_height,
            wave_speed,
            wave_pitch,
            texture_cyclical_interval,
        })
    }
}
