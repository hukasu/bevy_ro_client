mod components;
mod loader;

use bevy::{app::Plugin as BevyPlugin, asset::AssetApp};

pub use self::{components::Ground, loader::AssetLoader};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<components::Ground>()
            // Asset Loader
            .register_asset_loader(AssetLoader);
    }
}
