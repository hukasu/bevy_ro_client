use bevy::{
    prelude::{Deref, DerefMut, ReflectResource, Resource},
    reflect::Reflect,
};

#[derive(Debug, Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct GroundScale(pub f32);

impl Default for GroundScale {
    fn default() -> Self {
        GroundScale(1.)
    }
}
