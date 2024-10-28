use std::time::Duration;

use bevy::{
    asset::Handle, ecs::component::Component, prelude::ReflectComponent, reflect::Reflect,
    time::Timer,
};

use super::WaterPlaneMaterial;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct WaterPlane {
    pub(super) current_frame: usize,
    pub(super) frames: [Handle<WaterPlaneMaterial>; 32],
    pub(super) timer: Timer,
}

impl WaterPlane {
    pub fn new(textures: [Handle<WaterPlaneMaterial>; 32], cycle: i32) -> Self {
        Self {
            current_frame: 0,
            frames: textures,
            timer: Timer::new(
                Duration::from_secs_f32(cycle as f32 / 60.),
                bevy::time::TimerMode::Repeating,
            ),
        }
    }
}
