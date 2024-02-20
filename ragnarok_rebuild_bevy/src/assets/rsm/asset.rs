use bevy::{
    asset::{Asset as BevyAsset, Handle},
    reflect::TypePath,
    render::texture::Image,
    utils::HashMap,
};
use ragnarok_rebuild_common::assets::rsm;

#[derive(Debug, BevyAsset, TypePath)]
pub struct Asset {
    pub rsm: rsm::RSM,
    pub textures: HashMap<Box<str>, Box<[Handle<Image>]>>,
}
