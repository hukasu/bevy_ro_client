use bevy::prelude::{Entity, Resource};

#[derive(Debug, Resource)]
pub struct LoadingWorld {
    pub world: Entity,
}
