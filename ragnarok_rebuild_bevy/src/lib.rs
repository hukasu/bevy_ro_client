use bevy::app::Plugin;

pub mod assets;

pub struct RagnarokPlugin;

impl Plugin for RagnarokPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(assets::PluginGroup);
    }
}
