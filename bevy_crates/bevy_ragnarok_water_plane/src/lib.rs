//! [`WaterPlane`] are planes with a wave animation to replicate water

pub mod material;
pub mod plugin;

use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;

#[derive(Debug, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct WaterPlane {
    pub water_level: f32,
    pub water_type: i32,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_pitch: f32,
    pub texture_cyclical_interval: i32,
}

impl From<ragnarok_water_plane::WaterPlane> for WaterPlane {
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
