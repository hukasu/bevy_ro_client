pub mod assets;
pub mod events;
pub mod plugin;
pub mod relationships;

use std::borrow::Cow;

use bevy_asset::Handle;
use bevy_audio::AudioSource;
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;
use bevy_time::Timer;
use bevy_transform::components::Transform;

#[derive(Debug, Reflect, Component)]
#[reflect(Component)]
/// A [`World`] contains a Ground,
/// a [`Altitude`],
/// a [`DirectionalLight`](bevy_pbr::DirectionalLight),
/// multiple [`AnimatedProp`],
/// multiple [`PointLight`](bevy_pbr::PointLight),
/// and multiple [`EnvironmentalSounds`](crate::EnvironmentalSound)
pub struct World;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
/// Ground of the [`World`]
pub struct Ground {
    pub ground_path: Cow<'static, str>,
}

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
/// Tile information from a [`Gat`]
pub struct Altitude;

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// A diffuse light that illuminated the [`World`]
pub struct DiffuseLight;

#[derive(Debug, Default, Clone, Component, Reflect)]
#[reflect(Component)]
/// An animated prop that is part of the [`World`]
pub struct AnimatedProp {
    pub prop_path: Cow<'static, str>,
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

#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
/// The quad tree of the [`World`]
pub struct WorldQuadTree;
