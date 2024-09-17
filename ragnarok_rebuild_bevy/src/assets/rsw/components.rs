use bevy::{
    asset::Handle,
    audio::AudioSource,
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::Transform,
    reflect::Reflect,
    time::Timer,
};

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
/// A World contains a Ground, a [`DirectionalLight`](bevy::pbr::DirectionalLight), multiple [`AnimatedProp`],
/// multiple [`PointLight`](bevy::pbr::PointLight), and multiple [`EnvironmentalSound`]s
pub struct World;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// A diffuse light that illuminated the [`World`]
pub struct DiffuseLight;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// An animated prop that is part of the [`World`]
pub struct AnimatedProp {
    pub animation_type: i32,
    pub animation_speed: f32,
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Component marker for a entity that holds all the environment lights of a [`World`]
pub struct EnvironmentalLight {
    pub range: f32,
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Environmental sound that plays in the [`World`]
pub struct EnvironmentalSound {
    pub name: String,
    pub source: Handle<AudioSource>,
    pub position: Transform,
    pub volume: f32,
    pub range: f32,
    pub cycle: Timer,
}

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// Environmental effect that create particles in the [`World`]
pub struct EnvironmentalEffect;
