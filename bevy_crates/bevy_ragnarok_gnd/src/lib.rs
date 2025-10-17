pub mod assets;
mod components;
#[cfg(feature = "debug")]
mod debug;
mod loader;
mod material;
mod resources;

use bevy_asset::AssetApp;

use crate::assets::GndAsset;

pub use self::{
    components::Ground,
    loader::{AssetLoader, AssetLoaderSettings},
    resources::GroundScale,
};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Resources
        app.init_resource::<GroundScale>();

        // Asset Loader
        app.init_asset::<GndAsset>()
            .register_asset_loader(AssetLoader);

        // Material
        app.add_plugins(material::Plugin);

        // Types
        app.register_type::<Ground>().register_type::<GroundScale>();

        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);
    }
}
