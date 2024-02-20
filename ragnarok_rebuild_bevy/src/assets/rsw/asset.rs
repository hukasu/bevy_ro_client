use bevy::{
    asset::{Asset as BevyAsset, Handle, UntypedHandle},
    audio::AudioSource,
    reflect::TypePath,
    render::texture::Image,
    scene::Scene,
};

#[derive(Debug, BevyAsset, TypePath)]
pub struct Asset {
    pub rsw: super::RSW,
    pub ini_handle: Option<UntypedHandle>,
    pub gnd_handle: Option<UntypedHandle>,
    pub gat_handle: Option<UntypedHandle>,
    pub source_handle: Option<UntypedHandle>,
    pub rsm_handles: Vec<Handle<Scene>>,
    pub sound_handles: Vec<Handle<AudioSource>>,
    pub effect_handles: Vec<UntypedHandle>,
    pub water_textures: Option<[Handle<Image>; 32]>,
}
