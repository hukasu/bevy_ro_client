mod loader;

use bevy::asset::AssetApp;

pub use self::loader::Sprite;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Asset Loader
            .register_asset_loader(loader::AssetLoader)
            // Asset
            .init_asset::<loader::Sprite>();
    }
}
