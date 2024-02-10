use bevy::{
    asset::{Asset as BevyAsset, Handle, UntypedHandle},
    audio::AudioSource,
    reflect::TypePath,
};

#[derive(Debug, BevyAsset, TypePath)]
pub struct Asset {
    pub rsw: super::RSW,
    pub ini_handle: Option<UntypedHandle>,
    pub gnd_handle: Option<UntypedHandle>,
    pub gat_handle: Option<UntypedHandle>,
    pub source_handle: Option<UntypedHandle>,
    pub rsm_handles: Vec<UntypedHandle>,
    pub sound_handles: Vec<Handle<AudioSource>>,
    pub effect_handles: Vec<UntypedHandle>,
}
