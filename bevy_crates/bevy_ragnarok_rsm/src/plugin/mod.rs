mod loader;

use bevy_asset::AssetApp;
use loader::AssetLoader;

use crate::{Model, RsmMaterials, assets::RsmModel, materials::RsmMaterial};

pub struct Plugin {
    pub texture_path_prefix: std::path::PathBuf,
}

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Assets
        app.init_asset::<RsmModel>()
            .register_asset_reflect::<RsmModel>()
            .register_asset_loader(AssetLoader::new(self.texture_path_prefix.clone()));

        // Materials
        app.add_plugins(crate::materials::Plugin);

        // Types
        app.register_type::<Model>();
        app.register_type::<RsmMaterials>();

        #[cfg(feature = "debug")]
        app.add_plugins(crate::debug::Plugin);
    }
}
