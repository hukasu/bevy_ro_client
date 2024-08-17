pub mod assets;
#[cfg(feature = "audio")]
pub mod audio;
#[cfg(feature = "debug")]
pub mod debug;
pub mod world;

use bevy::app::Plugin;

pub struct RagnarokPlugin;

impl Plugin for RagnarokPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(assets::PluginGroup)
            .add_plugins(world::WorldPlugin);

        #[cfg(feature = "audio")]
        app.add_plugins(audio::AudioPlugin);

        #[cfg(feature = "debug")]
        app.add_plugins(debug::DebugPlugin);
    }
}
