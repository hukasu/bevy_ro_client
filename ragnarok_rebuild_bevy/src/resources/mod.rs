use bevy::{
    prelude::{Deref, DerefMut, ReflectResource, Resource, Transform},
    reflect::Reflect,
};

#[derive(Debug, Default, Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct WorldTransform(pub Transform);
