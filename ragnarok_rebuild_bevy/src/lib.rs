use bevy::app::Plugin;

pub mod assets;
#[cfg(feature = "debug")]
pub mod debug;

pub struct RagnarokPlugin;

impl Plugin for RagnarokPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(assets::PluginGroup);
        #[cfg(feature = "debug")]
        app.add_plugins(debug::DebugPlugin);
    }
}
