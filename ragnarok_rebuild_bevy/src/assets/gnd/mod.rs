mod components;
mod loader;
mod material;
mod resources;

use bevy::{app::Plugin as BevyPlugin, asset::AssetApp};

pub use self::{
    components::Ground,
    loader::{AssetLoader, AssetLoaderSettings},
    resources::GroundScale,
};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<Ground>()
            .register_type::<GroundScale>()
            // Resources
            .init_resource::<GroundScale>()
            // Asset Loader
            .register_asset_loader(AssetLoader)
            // Material
            .add_plugins(material::Plugin);
    }
}
