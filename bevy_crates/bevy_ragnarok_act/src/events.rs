use bevy_animation::AnimationEvent;
use bevy_asset::Handle;
use bevy_audio::AudioSource;
use bevy_ecs::name::Name;

#[derive(Debug, Clone, AnimationEvent)]
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
