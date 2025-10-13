pub mod plugin;
// TODO remove pub after organizing the debug systems
mod audio;
mod camera;
pub mod entities;
mod loading_screen;
pub mod states;
pub mod world;

use bevy::prelude::Component;

#[derive(Debug, Component)]
pub struct Game;
