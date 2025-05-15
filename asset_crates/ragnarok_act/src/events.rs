use bevy_asset::Handle;
use bevy_audio::AudioSource;
use bevy_ecs::{event::Event, name::Name};

#[derive(Debug, Clone, Event)]
pub struct ActorSound {
    name: Name,
    sound: Handle<AudioSource>,
}

impl ActorSound {
    pub fn new(name: Name, sound: Handle<AudioSource>) -> Self {
        Self { name, sound }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn sound(&self) -> &Handle<AudioSource> {
        &self.sound
    }
}
