use std::borrow::Cow;

use bevy::{
    asset::Handle,
    audio::AudioSource,
    prelude::{Event, Transform},
};

#[derive(Debug, Event)]
pub struct PlayBgm {
    pub track: Cow<'static, str>,
}

#[derive(Debug, Event)]
pub struct PlaySound {
    pub name: String,
    pub track: Handle<AudioSource>,
    pub position: Transform,
    pub volume: f32,
    pub range: f32,
}
