mod components;
mod loader;

use bevy::{
    app::{Plugin as BevyPlugin, Update},
    asset::{AssetApp, Handle},
    core::Name,
    prelude::{Added, AnimationPlayer, Children, OnAdd, Query, Trigger, With},
    scene::Scene,
};

pub use self::{components::World, loader::AssetLoader};
pub use ragnarok_rebuild_assets::rsw::Error;

use super::rsm;

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<components::Ground>()
            .register_type::<components::Models>()
            .register_type::<components::WorldModel>()
            .register_type::<components::EnvironmentalLights>()
            .register_type::<components::EnvironmentalSounds>()
            .register_type::<components::EnvironmentalSound>()
            // Loader
            .register_asset_loader(AssetLoader);
    }
}
