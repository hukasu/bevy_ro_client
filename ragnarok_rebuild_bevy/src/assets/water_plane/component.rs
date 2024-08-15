use std::time::Duration;

use bevy::{asset::Handle, ecs::component::Component, render::texture::Image, time::Timer};

#[derive(Debug, Component)]
pub struct WaterPlane {
    pub current_frame: usize,
    pub frames: [Handle<Image>; 32],
    pub timer: Timer,
}

impl WaterPlane {
    pub fn new(textures: [Handle<Image>; 32], cycle: i32) -> Self {
        Self {
            current_frame: 0,
            frames: textures,
            timer: Timer::new(
                Duration::from_secs_f32((1. / 60.) * cycle as f32),
                bevy::time::TimerMode::Repeating,
            ),
        }
    }
}
