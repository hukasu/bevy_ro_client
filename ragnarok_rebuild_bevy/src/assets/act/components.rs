use bevy::{
    asset::Handle,
    prelude::{Component, Image, ReflectComponent, ReflectDefault},
    reflect::Reflect,
    time::Timer,
};

use crate::assets::spr::Sprite;

use super::Animation;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Actor {
    pub act: Handle<Animation>,
    pub sprite: Handle<Sprite>,
    pub palette: Handle<Image>,
    pub facing: ActorFacing,
    pub clip: usize,
    pub frame: usize,
    pub timer: Timer,
}

#[derive(Debug, Default, Clone, Copy, Reflect)]
#[reflect(Default)]
pub enum ActorFacing {
    #[default]
    SouthWest,
    South,
    SouthEast,
    East,
    NorthEast,
    North,
    NorthWest,
    West,
}

#[derive(Debug, Component)]
pub struct LoadingActor;
