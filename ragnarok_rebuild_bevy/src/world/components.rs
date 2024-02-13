use bevy::ecs::component::Component;

#[derive(Debug, Component)]
/// Represents a map
pub struct World;

#[derive(Debug, Component)]
/// Component marker for a entity that holds all the sounds of a [World]
pub struct Sounds;

#[derive(Debug, Component)]
/// Component marker for a entity that holds all the models of a [World]
pub struct Models;
