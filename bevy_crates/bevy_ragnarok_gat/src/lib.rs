mod assets;
#[cfg(feature = "debug")]
pub mod debug;
mod loader;
pub mod plugin;

use std::borrow::Borrow;

use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;

#[derive(Debug, Clone, Copy, Reflect, Component)]
#[reflect(Component)]
pub struct Tile {
    pub bottom_left: f32,
    pub bottom_right: f32,
    pub top_left: f32,
    pub top_right: f32,
    // TODO change to gat::Tile when Bevy implements foreign type reflection
    pub tile_type: u8,
    pub is_water_tile: bool,
}

impl<T: Borrow<ragnarok_gat::Tile>> From<T> for Tile {
    fn from(tile: T) -> Self {
        let tile = tile.borrow();
        Self {
            bottom_left: tile.bottom_left_altitude(),
            bottom_right: tile.bottom_right_altitude(),
            top_left: tile.top_left_altitude(),
            top_right: tile.top_right_altitude(),
            tile_type: tile.tile_type(),
            is_water_tile: tile.is_water_tile(),
        }
    }
}
