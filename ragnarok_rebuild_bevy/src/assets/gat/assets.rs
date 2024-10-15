use bevy::{asset::Asset, reflect::TypePath};
use ragnarok_rebuild_assets::gat;

#[allow(dead_code)]
#[derive(Debug, Asset, TypePath)]
pub struct Gat(pub gat::Gat);
