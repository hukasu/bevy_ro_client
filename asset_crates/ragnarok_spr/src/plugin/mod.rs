mod loader;

use bevy_asset::AssetApp;

use crate::{assets::SpriteImages, components::Sprite, material};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Types
            .register_type::<Sprite>()
            // Asset Loader
            .register_asset_loader(loader::AssetLoader)
            // Asset
            .init_asset::<SpriteImages>()
            .register_asset_reflect::<SpriteImages>()
            // MaterialPlugin
            .add_plugins(material::Plugin);
    }
}
