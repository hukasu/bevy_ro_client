mod components;
mod loader;

use bevy::{app::Plugin as BevyPlugin, asset::AssetApp};

pub use ragnarok_rebuild_assets::rsw::Error;

pub use self::{components::EnvironmentalLights, loader::AssetLoader};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<components::Models>()
            .register_type::<components::WorldModel>()
            .register_type::<components::EnvironmentalLights>()
            .register_type::<components::EnvironmentalSounds>()
            .register_type::<components::EnvironmentalSound>()
            // Loader
            .register_asset_loader(AssetLoader);
    }
}
