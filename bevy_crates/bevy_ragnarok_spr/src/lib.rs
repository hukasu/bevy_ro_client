//! Builds Ragnarok Online's Spr files to be used in Bevy

pub mod assets;
pub mod material;
pub mod plugin;

use bevy_asset::Handle;
use bevy_ecs::component::Component;
use bevy_reflect::Reflect;

use crate::assets::SpriteImages;

#[derive(Debug, Component, Reflect)]
pub struct Sprite(pub Handle<SpriteImages>);
