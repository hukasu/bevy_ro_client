use bevy_animation::graph::AnimationGraph;
use bevy_asset::{Asset, Handle};
use bevy_reflect::Reflect;
use bevy_scene::Scene;

#[derive(Debug, Asset, Reflect)]
pub struct ActorAnimations {
    pub animation_graph: Handle<AnimationGraph>,
    pub scene: Handle<Scene>,
}
