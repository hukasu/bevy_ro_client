use std::borrow::Cow;

use bevy_asset::Handle;
use bevy_audio::AudioSource;
use bevy_ecs::event::Event;
use bevy_transform::components::Transform;

#[derive(Debug, Event)]
pub struct LoadWorld {
    pub world: Cow<'static, str>,
}

#[derive(Debug, Event)]
pub struct UnloadWorld;

#[derive(Debug, Event)]
pub struct WorldLoaded;

#[derive(Debug, Event)]
pub struct WorldSound {
    pub name: String,
    pub track: Handle<AudioSource>,
    pub position: Transform,
    pub volume: f32,
    pub range: f32,
}
