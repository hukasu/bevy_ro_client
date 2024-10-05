use bevy::{
    asset::Handle,
    prelude::{Component, ReflectComponent, ReflectDefault},
    reflect::Reflect,
    time::Timer,
};

use super::Animation;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Actor {
    pub act: Handle<Animation>,
    pub facing: ActorFacing,
    pub clip: usize,
    pub frame: usize,
    pub timer: Timer,
}

#[derive(Debug, Default, Clone, Copy, Reflect)]
#[reflect(Default)]
pub enum ActorFacing {
    #[default]
    South,
    SouthWest,
    West,
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
}

#[derive(Debug, Component)]
pub struct LoadingActor;
