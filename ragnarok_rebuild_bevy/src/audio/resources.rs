use bevy::{
    prelude::{ReflectResource, Resource},
    reflect::Reflect,
};

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct AudioSettings {
    pub bgm_volume: f32,
    pub effects_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        AudioSettings {
            bgm_volume: 0.2,
            effects_volume: 0.2,
        }
    }
}
