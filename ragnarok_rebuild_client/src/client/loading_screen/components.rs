use bevy::{
    prelude::{Component, ReflectComponent},
    reflect::Reflect,
};

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct LoadingScreen;
