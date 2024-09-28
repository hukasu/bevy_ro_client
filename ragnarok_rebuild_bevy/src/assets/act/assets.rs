use bevy::{
    asset::{Asset, Handle},
    audio::AudioSource,
    color::Color,
    math::{IVec2, Vec2},
    reflect::TypePath,
};

#[derive(Debug, Asset, TypePath)]
pub struct Animation {
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
    Indexed(usize),
    TrueColor(usize),
}

#[derive(Debug)]
pub enum AnimationEvent {
    Attack,
    Sound(Handle<AudioSource>),
}
