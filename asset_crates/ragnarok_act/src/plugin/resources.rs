use std::ops::{Deref, DerefMut};

use bevy_ecs::{entity::Entity, resource::Resource};

#[derive(Debug, Default, Resource)]
pub struct ActorSceneQueue(pub Vec<Entity>);

impl Deref for ActorSceneQueue {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ActorSceneQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
