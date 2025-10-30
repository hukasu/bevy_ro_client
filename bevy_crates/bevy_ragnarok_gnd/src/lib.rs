pub mod assets;
#[cfg(feature = "debug")]
mod debug;
mod material;
pub mod plugin;

use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;

/// An entity that represents the ground mesh of the map
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Ground {
    pub width: u32,
    pub height: u32,
    /// This is the unit length of a ground cube.
    /// Each ground cube contains 2x2 tiles.
    pub scale: f32,
}

/// Represents a ground cube
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Cube;
