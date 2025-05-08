use bevy_animation::{AnimationClip, graph::AnimationNodeIndex};
use bevy_asset::Handle;
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;

use crate::materials::RsmMaterial;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Model {
    pub animation: Option<ModelAnimation>,
}

#[derive(Debug, Reflect)]
pub struct ModelAnimation {
    pub animation: Handle<AnimationClip>,
    pub animation_node_index: AnimationNodeIndex,
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct ModelInvertedMaterial(pub Handle<RsmMaterial>);
