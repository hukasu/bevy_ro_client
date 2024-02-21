use std::io::Read;

use crate::reader_ext::ReaderExt;

pub struct GroundMeshCube {
    pub bottom_left_height: f32,
    pub bottom_right_height: f32,
    pub top_left_height: f32,
    pub top_right_height: f32,
    pub upwards_facing_surface: i32,
    pub north_facing_surface: i32,
    pub east_facing_surface: i32,
}

impl GroundMeshCube {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let bottom_left_height = reader.read_le_f32()?;
        let bottom_right_height = reader.read_le_f32()?;
        let top_left_height = reader.read_le_f32()?;
        let top_right_height = reader.read_le_f32()?;
        let upwards_facing_surface = reader.read_le_i32()?;
        let north_facing_surface = reader.read_le_i32()?;
        let east_facing_surface = reader.read_le_i32()?;

        Ok(Self {
            bottom_left_height,
            bottom_right_height,
            top_left_height,
            top_right_height,
            upwards_facing_surface,
            north_facing_surface,
            east_facing_surface,
        })
    }
}
