use bevy_asset::AssetApp;

use crate::Palette;

mod loader;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Types
            .register_type::<Palette>()
            // Asset loader
            .register_asset_loader(loader::AssetLoader);
    }
}
