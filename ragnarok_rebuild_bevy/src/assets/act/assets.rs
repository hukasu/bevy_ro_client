#[cfg(feature = "audio")]
use bevy::audio::AudioSource;
use bevy::{
    asset::{Asset, Handle},
    color::Color,
    math::{IVec2, Vec2},
    prelude::Image,
    reflect::TypePath,
};

use crate::assets::spr;

#[derive(Debug, Asset, TypePath)]
pub struct Animation {
    pub sprite: Handle<spr::Sprite>,
    pub palette: Handle<Image>,
    pub clips: Box<[AnimationClip]>,
}

#[derive(Debug)]
pub struct AnimationClip {
    pub frame_time: f32,
    pub frames: Box<[AnimationFrame]>,
}

#[derive(Debug)]
pub struct AnimationFrame {
    pub layers: Box<[AnimationLayer]>,
    pub anchors: Box<[IVec2]>,
    pub event: Option<AnimationEvent>,
}

#[derive(Debug)]
pub struct AnimationLayer {
    pub origin: IVec2,
    pub sprite: AnimationLayerSprite,
    pub is_flipped: bool,
    pub tint: Color,
    pub scale: Vec2,
    /// Rotation in radians
    pub rotation: f32,
    pub image_size: IVec2,
}

#[derive(Debug)]
pub enum AnimationLayerSprite {
    Indexed(Handle<spr::SprIndexedMaterial>),
    TrueColor(Handle<spr::SprTrueColorMaterial>),
}

#[derive(Debug)]
pub enum AnimationEvent {
    Attack,
    #[cfg(feature = "audio")]
    Sound(Handle<AudioSource>),
}
