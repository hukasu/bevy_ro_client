use bevy::{
    asset::{Asset as BevyAsset, Handle},
    reflect::TypePath,
    render::texture::Image,
    utils::HashMap,
};

#[derive(Debug, BevyAsset, TypePath)]
pub struct Asset {
    pub rsm: super::RSM,
    pub textures: HashMap<Box<str>, Box<[Handle<Image>]>>,
}
