//! Builds Ragnarok Online's Spr files to be used in Bevy

pub mod material;
pub mod plugin;

use bevy_asset::{Asset, Handle};
use bevy_ecs::component::Component;
use bevy_image::Image;
use bevy_reflect::Reflect;
use bevy_render::storage::ShaderStorageBuffer;

/// Holds an handles to the assets generated from a [`Spr`](ragnarok_spr::Spr).
#[derive(Debug, Component, Reflect)]
pub struct Sprite(pub Handle<SpriteImages>);

/// Assets generated from a [`Spr`](ragnarok_spr::Spr).
#[derive(Debug, Asset, Reflect)]
pub struct SpriteImages {
    /// Texture atlas with indices into the palette
    pub indexed_sprites: Handle<Image>,
    /// Uvs into the indices texture atlas
    pub indexed_sprites_uvs: Handle<ShaderStorageBuffer>,
    /// True color sprite texture atlas
    pub true_color_sprites: Handle<Image>,
    /// Uvs into the true color texture atlas
    pub true_color_sprites_uvs: Handle<ShaderStorageBuffer>,
    /// Default sprite palette
    pub palette: Handle<Image>,
}

impl SpriteImages {
    /// Stride of each item in the Uvs [`ShaderStorageBuffer`]
    pub const UV_STRIDE: usize = 4 * 4;
}
