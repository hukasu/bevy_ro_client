pub mod assets;
#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "debug")]
pub mod debug;
pub mod materials;
mod resources;
pub mod tables;

use bevy::app::Plugin;

pub use self::resources::WorldTransform;

pub struct RagnarokPlugin;

impl Plugin for RagnarokPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(assets::PluginGroup)
            .add_plugins(tables::TablePlugins)
            .add_plugins(materials::PluginGroup);

        #[cfg(feature = "audio")]
        app.add_plugins(audio::AudioPlugin);

        #[cfg(feature = "debug")]
        app.add_plugins(debug::DebugPlugin);

        app.register_type::<WorldTransform>()
            .init_resource::<WorldTransform>();
    }
}
