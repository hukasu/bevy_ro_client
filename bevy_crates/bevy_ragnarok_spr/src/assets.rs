use bevy_asset::{Asset, Handle};
use bevy_image::Image;
use bevy_reflect::Reflect;

#[derive(Debug, Asset, Reflect)]
pub struct SpriteImages {
    pub indexed_sprites: Vec<Handle<Image>>,
    pub true_color_sprites: Vec<Handle<Image>>,
    pub palette: Handle<Image>,
}
