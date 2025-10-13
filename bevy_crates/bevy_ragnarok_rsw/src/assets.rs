use bevy_asset::{Asset, Handle};
use bevy_reflect::Reflect;
use bevy_scene::Scene;

/// [`Scene`] representation of a [`Rsw`](ragnarok_rsw::Rsw).
#[derive(Debug, Asset, Reflect)]
pub struct RswAsset {
    pub scene: Handle<Scene>,
}
