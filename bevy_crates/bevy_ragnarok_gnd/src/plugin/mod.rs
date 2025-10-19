mod loader;

use bevy_asset::AssetApp;

#[cfg(feature = "debug")]
use crate::debug;
use crate::{Ground, assets::GndAsset, material, plugin::loader::AssetLoader};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Asset Loader
        app.init_asset::<GndAsset>()
            .register_asset_loader(AssetLoader);

        // Material
        app.add_plugins(material::Plugin);

        // Types
        app.register_type::<Ground>();

        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);
    }
}
