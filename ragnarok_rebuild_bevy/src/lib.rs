pub mod assets;
#[cfg(feature = "audio")]
pub mod audio;
pub mod helper;
mod resources;
pub mod tables;

use bevy::app::Plugin;

pub use self::resources::WorldTransform;

pub struct RagnarokPlugin;

impl Plugin for RagnarokPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(assets::PluginGroup)
            .add_plugins(tables::TablePlugins);

        #[cfg(feature = "audio")]
        app.add_plugins(audio::AudioPlugin);

        app.register_type::<WorldTransform>()
            .init_resource::<WorldTransform>();
    }
}
