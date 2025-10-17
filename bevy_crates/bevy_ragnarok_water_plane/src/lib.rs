//! [`WaterPlane`] are planes with a wave animation to replicate water

pub mod material;
pub mod plugin;

use std::time::Duration;

use bevy_asset::Handle;
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::Reflect;
use bevy_time::{Timer, TimerMode};

use crate::material::WaterPlaneMaterial;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct WaterPlane {
    current_frame: usize,
    frames: [Handle<WaterPlaneMaterial>; 32],
    timer: Timer,
}

impl WaterPlane {
    pub fn new(textures: [Handle<WaterPlaneMaterial>; 32], cycle: i32) -> Self {
        Self {
            current_frame: 0,
            frames: textures,
            timer: Timer::new(
                Duration::from_secs_f32(cycle as f32 / 60.),
                TimerMode::Repeating,
            ),
        }
    }
}
