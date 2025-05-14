use bevy_asset::AssetApp;

mod loader;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.register_asset_loader(loader::AssetLoader);
    }
}
