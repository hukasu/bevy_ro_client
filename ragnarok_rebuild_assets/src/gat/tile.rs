use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    top_left_altitude: f32,
    top_right_altitude: f32,
    bottom_left_altitude: f32,
    bottom_right_altitude: f32,
    tile_type: u8,
    is_water_tile: bool,
}

impl Tile {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let top_left_altitude = reader.read_le_f32()?;
        let top_right_altitude = reader.read_le_f32()?;
        let bottom_left_altitude = reader.read_le_f32()?;
        let bottom_right_altitude = reader.read_le_f32()?;

        let tile_type = reader.read_u8()?;
        let _ = reader.read_u8()?;
        let _ = reader.read_u8()?;
        let is_water_tile = reader.read_u8()? == 0x80;

        Ok(Self {
            top_left_altitude,
            top_right_altitude,
            bottom_left_altitude,
            bottom_right_altitude,
            tile_type,
            is_water_tile,
        })
    }

    pub fn top_left_altitude(&self) -> f32 {
        self.top_left_altitude
    }

    pub fn top_right_altitude(&self) -> f32 {
        self.top_right_altitude
    }

    pub fn bottom_left_altitude(&self) -> f32 {
        self.bottom_left_altitude
    }

    pub fn bottom_right_altitude(&self) -> f32 {
        self.bottom_right_altitude
    }

    pub fn tile_type(&self) -> u8 {
        self.tile_type
    }

    pub fn is_water_tile(&self) -> bool {
        self.is_water_tile
    }
}

// TODO change type of member `tile_type` on `Tile` after Bevy implements
// foreign type reflection
pub enum TileType {
    WalkableBlock,
    NonWalkableBlock,
    NonWalkableWater,
    WalkableWater,
    SnipeableNonWalkableWater,
    SnipeableCliff,
    Cliff,
    Unknown,
}
