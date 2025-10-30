//! Deals with loading Rsw and its dependencies

pub mod plugin;

use std::{borrow::Cow, fmt::Debug};

use bevy::{
    ecs::{component::Component, entity::Entity, event::Event},
    state::state::States,
};

#[derive(Debug, Component)]
#[relationship(relationship_target=GameOfWorld)]
pub struct WorldOfGame(Entity);

#[derive(Debug, Component)]
#[relationship_target(relationship=WorldOfGame)]
pub struct GameOfWorld(Entity);

/// Event to trigger a map change
#[derive(Debug, Event)]
pub struct ChangeMap {
    /// Destination map. Must have `.rsw` extension.
    pub map: Cow<'static, str>,
}

impl ChangeMap {
    /// Creates a new instance of [`ChangeMap`]
    pub fn new(map: impl Into<Cow<'static, str>>) -> Self {
        Self { map: map.into() }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum MapChangeStates {
    #[default]
    Unloaded,
    LoadingAsset,
    LoadingGround,
    LoadingAltitude,
    LoadingRswWaterPlane,
    LoadingModels,
    Loaded,
}
