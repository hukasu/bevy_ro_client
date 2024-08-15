mod loader;

use bevy::{app::Plugin as BevyPlugin, asset::AssetApp};

pub use self::loader::AssetLoader;

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_loader(AssetLoader);
    }
}
