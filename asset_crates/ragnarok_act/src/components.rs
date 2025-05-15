use bevy_animation::{
    AnimationPlayer,
    prelude::{Animatable, AnimationTransitions},
};
use bevy_asset::Handle;
use bevy_color::LinearRgba;
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_reflect::{Reflect, prelude::ReflectDefault};
use bevy_render::view::Visibility;
use bevy_transform::components::Transform;

use crate::assets::ActorAnimations;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
#[require(ActorFacing, Transform, Visibility)]
pub struct Actor {
    pub actor: Handle<ActorAnimations>,
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Default, Component)]
#[repr(u8)]
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

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Clone, Component)]
#[require(AnimationPlayer, AnimationTransitions)]
pub struct ActorPlayer;

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Clone, Component)]
pub struct ActorLayer {
    pub active: bool,
    pub spritesheet_index: SpritesheetIndex,
    pub uv_flip: bool,
    pub tint: LinearRgba,
}

#[derive(Debug, Default, Clone, Copy, Component, Reflect)]
#[reflect(Clone, Component)]
pub struct ActorAnchor;

#[derive(Debug, Clone, Copy, Reflect)]
#[reflect(Clone)]
pub enum SpritesheetIndex {
    Indexed(usize),
    TrueColor(usize),
    None,
}

impl Default for SpritesheetIndex {
    fn default() -> Self {
        Self::Indexed(0)
    }
}

impl Animatable for SpritesheetIndex {
    fn blend(mut inputs: impl Iterator<Item = bevy_animation::prelude::BlendInput<Self>>) -> Self {
        unsafe { inputs.next().unwrap_unchecked().value }
    }

    fn interpolate(a: &Self, _b: &Self, _time: f32) -> Self {
        *a
    }
}
