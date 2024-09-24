use bevy::asset::AssetApp;

mod loader;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_loader(loader::AssetLoader);
    }
}
