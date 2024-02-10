use bevy::ecs::{component::Component, entity::Entity};

#[derive(Debug, Component)]
pub struct World {
    pub world_sounds: Entity,
}

#[derive(Debug, Component)]
pub struct Sounds;
