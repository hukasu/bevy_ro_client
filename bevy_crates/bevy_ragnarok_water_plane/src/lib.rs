//! [`WaterPlane`] are planes with a wave animation to replicate water

pub mod material;
pub mod plugin;

use bevy_asset::{Asset, Handle, ReflectAsset};
use bevy_derive::Deref;
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;

#[derive(Debug, Clone, Component, Reflect, Deref)]
#[reflect(Component)]
pub struct WaterPlane(Handle<WaterPlaneAsset>);

impl<T: Into<Handle<WaterPlaneAsset>>> From<T> for WaterPlane {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct WaterPlaneBuilder {
    /// The width in tiles of the [`WaterPlane`], each tiles occupies 4x4 tiles
    pub width: u32,
    /// The height in tiles of the [`WaterPlane`], each tiles occupies 4x4 tiles
    pub height: u32,
    /// Settings of the [`WaterPlane`]
    pub water_plane: Handle<WaterPlaneAsset>,
}

#[derive(Debug, Clone, Copy, Asset, Reflect)]
#[reflect(Asset)]
pub struct WaterPlaneAsset {
    pub water_level: f32,
    pub water_type: i32,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_pitch: f32,
    pub texture_cyclical_interval: i32,
}

impl From<ragnarok_water_plane::WaterPlane> for WaterPlaneAsset {
    fn from(value: ragnarok_water_plane::WaterPlane) -> Self {
        Self {
            water_level: value.water_level,
            water_type: value.water_type,
            wave_height: value.wave_height,
            wave_speed: value.wave_speed,
            wave_pitch: value.wave_pitch,
            texture_cyclical_interval: value.texture_cyclical_interval,
        }
    }
}
