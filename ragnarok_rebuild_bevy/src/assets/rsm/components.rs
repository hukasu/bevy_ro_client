use bevy::{
    animation::AnimationClip,
    asset::Handle,
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::AnimationNodeIndex,
    reflect::Reflect,
};

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
