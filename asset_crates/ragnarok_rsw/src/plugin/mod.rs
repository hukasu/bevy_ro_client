// #[cfg(feature = "debug")]
// mod debug;
mod loader;
// mod quad_tree;

use std::path::PathBuf;

use bevy_asset::AssetApp;

use crate::{assets::RswWorld, components};

use self::loader::AssetLoader;

pub struct Plugin {
    /// Prefix for .rsm files
    pub model_path_prefix: PathBuf,
    /// Prefix for .gnd files
    pub ground_path_prefix: PathBuf,
    /// Prefix for .gat files
    pub altitude_path_prefix: PathBuf,
    /// Prefix for .wav files
    pub sound_path_prefix: PathBuf,
}

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Assets
            .init_asset::<RswWorld>()
            .register_asset_reflect::<RswWorld>()
            // Register Types
            .register_type::<components::World>()
            .register_type::<components::DiffuseLight>()
            .register_type::<components::AnimatedProp>()
            .register_type::<components::EnvironmentalLight>()
            .register_type::<components::EnvironmentalEffect>()
            .register_type::<components::EnvironmentalSound>()
            // Register AssetLoader
            .register_asset_loader(AssetLoader {
                model_path_prefix: self.model_path_prefix.clone(),
                ground_path_prefix: self.ground_path_prefix.clone(),
                altitude_path_prefix: self.altitude_path_prefix.clone(),
                sound_path_prefix: self.sound_path_prefix.clone(),
            });

        // #[cfg(feature = "debug")]
        // app.add_plugins(debug::Plugin);
    }
}
