use bevy_asset::{Asset, Handle};
use bevy_reflect::Reflect;
use bevy_scene::Scene;

#[derive(Debug, Asset, Reflect)]
pub struct RswWorld {
    pub scene: Handle<Scene>,
}
