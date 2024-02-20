mod asset;
mod loader;

use bevy::{app::Plugin as BevyPlugin, asset::AssetApp};

pub use self::{asset::Asset, loader::AssetLoader};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<Asset>().register_asset_loader(AssetLoader);
    }
}
