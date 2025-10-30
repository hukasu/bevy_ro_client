pub mod plugin;
// TODO remove pub after organizing the debug systems
mod audio;
mod camera;
pub mod entities;
mod loading_screen;
pub mod world;

use bevy::{prelude::Component, state::state::ComputedStates};

use crate::client::world::MapChangeStates;

#[derive(Debug, Component)]
pub struct Game;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameStates {
    Login,
    MapChange,
    Game,
}

impl ComputedStates for GameStates {
    type SourceStates = MapChangeStates;

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            MapChangeStates::Unloaded => Some(Self::Login),
            MapChangeStates::Loaded => Some(Self::Game),
            _ => Some(Self::MapChange),
        }
    }
}
