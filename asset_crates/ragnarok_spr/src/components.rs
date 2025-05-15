use bevy_asset::Handle;
use bevy_ecs::component::Component;
use bevy_reflect::Reflect;

use crate::assets::SpriteImages;

#[derive(Debug, Component, Reflect)]
pub struct Sprite(pub Handle<SpriteImages>);
