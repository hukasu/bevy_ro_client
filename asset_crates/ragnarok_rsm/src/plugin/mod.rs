#[cfg(feature = "debug")]
mod debug;
pub(crate) mod loader;

use bevy_asset::AssetApp;
use loader::AssetLoader;

use crate::Rsm;

pub struct Plugin {
    pub texture_path_prefix: std::path::PathBuf,
}

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Types
            .register_type::<super::components::Model>();

        // Assets
        app.init_asset::<Rsm>()
            .register_asset_loader(AssetLoader::new(self.texture_path_prefix.clone()));

        // Materials
        app.add_plugins(super::materials::Plugin);

        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);
    }
}
