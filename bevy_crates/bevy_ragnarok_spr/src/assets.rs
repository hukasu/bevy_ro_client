use bevy_asset::{Asset, Handle};
use bevy_image::Image;
use bevy_reflect::Reflect;
use bevy_render::storage::ShaderStorageBuffer;

#[derive(Debug, Asset, Reflect)]
pub struct SpriteImages {
    pub indexed_sprites: Handle<Image>,
    pub indexed_sprites_uvs: Handle<ShaderStorageBuffer>,
    pub true_color_sprites: Handle<Image>,
    pub true_color_sprites_uvs: Handle<ShaderStorageBuffer>,
    pub palette: Handle<Image>,
}

impl SpriteImages {
    pub const UV_STRIDE: usize = 4 * 4;
}
