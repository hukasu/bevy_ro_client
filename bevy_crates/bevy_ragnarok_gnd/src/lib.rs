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
    /// This is the unit length of a ground cube.
    /// Each ground cube contains 2x2 tiles.
    scale: f32,
}
