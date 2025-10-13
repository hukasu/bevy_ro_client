// #[cfg(feature = "debug")]
// mod debug;
mod loader;

use std::path::PathBuf;

use bevy_asset::AssetApp;

use crate::{
    AnimatedProp, DiffuseLight, EnvironmentalEffect, EnvironmentalLight, EnvironmentalSound,
    assets::RswAsset,
};

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
            .init_asset::<RswAsset>()
            .register_asset_reflect::<RswAsset>()
            // Register Types
            .register_type::<DiffuseLight>()
            .register_type::<AnimatedProp>()
            .register_type::<EnvironmentalLight>()
            .register_type::<EnvironmentalEffect>()
            .register_type::<EnvironmentalSound>()
            // Register AssetLoader
            .register_asset_loader(AssetLoader {
                ground_path_prefix: self.ground_path_prefix.clone(),
                altitude_path_prefix: self.altitude_path_prefix.clone(),
                sound_path_prefix: self.sound_path_prefix.clone(),
            });

        // #[cfg(feature = "debug")]
        // app.add_plugins(debug::Plugin);
    }
}
