use bevy::{
    animation::AnimationClip,
    asset::Handle,
    ecs::{component::Component, reflect::ReflectComponent},
    reflect::Reflect,
};

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct Model {
    pub animation: Handle<AnimationClip>,
}
