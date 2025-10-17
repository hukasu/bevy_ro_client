use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{reflect::ReflectResource, resource::Resource};
use bevy_reflect::Reflect;

#[derive(Debug, Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct GroundScale(pub f32);

impl Default for GroundScale {
    fn default() -> Self {
        GroundScale(1.)
    }
}
