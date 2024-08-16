use bevy::{
    asset::Handle,
    audio::AudioSource,
    ecs::{component::Component, reflect::ReflectComponent},
    reflect::Reflect,
};

#[derive(Debug, Component)]
/// Represents a map
pub struct World;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Component marker for the ground of a [World]
pub struct Ground;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Component marker for a entity that holds all the models of a [World]
pub struct Models;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// An animated prop that is part of the [World]
pub struct WorldModel {
    pub animation_type: i32,
    pub animation_speed: f32,
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Component marker for a entity that holds all the environment lights of a [World]
pub struct EnvironmentalLights;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Component marker for a entity that holds all the sounds of a [World]
pub struct EnvironmentalSounds;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Environmental sound that plays in the [World]
pub struct EnvironmentalSound {
    pub source: Handle<AudioSource>,
    pub volume: f32,
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Component marker for the water plane of a [World]
pub struct WaterPlane;
