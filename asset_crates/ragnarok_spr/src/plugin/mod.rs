mod loader;

use bevy_asset::AssetApp;

use crate::material;

pub use self::loader::Sprite;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Asset Loader
            .register_asset_loader(loader::AssetLoader)
            // Asset
            .init_asset::<loader::Sprite>()
            // MaterialPlugin
            .add_plugins(material::Plugin);
    }
}
