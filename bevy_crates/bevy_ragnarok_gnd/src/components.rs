use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;

/// An entity that represents the ground mesh of the map
#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Ground;
