use bevy::app::Plugin;

pub mod assets;
pub mod components;
pub mod water_plane;
pub mod websocket;

pub struct RagnarokPlugin;

impl Plugin for RagnarokPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(assets::PluginGroup)
            .add_plugins(water_plane::Plugin);
    }
}
