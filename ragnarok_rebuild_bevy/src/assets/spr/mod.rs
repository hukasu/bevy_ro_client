mod loader;
mod material;

use bevy::asset::AssetApp;

pub use self::{
    loader::Sprite,
    material::{SprIndexedMaterial, SprTrueColorMaterial, SprUniform},
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Asset Loader
            .register_asset_loader(loader::AssetLoader)
            // Asset
            .init_asset::<loader::Sprite>()
            // MaterialPlugin
            .add_plugins(material::Plugin);
    }
}
