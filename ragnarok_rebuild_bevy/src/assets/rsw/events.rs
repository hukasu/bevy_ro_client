use std::borrow::Cow;

use bevy::prelude::Event;

#[derive(Debug, Event)]
pub struct LoadWorld {
    pub world: Cow<'static, str>,
}

#[derive(Debug, Event)]
pub struct UnloadWorld;

#[derive(Debug, Event)]
pub struct WorldLoaded;
