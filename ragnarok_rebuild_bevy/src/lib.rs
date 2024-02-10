use bevy::app::Plugin;

pub mod assets;
pub mod components;
pub mod websocket;
pub mod world;

pub struct RagnarokPlugin;

impl Plugin for RagnarokPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(assets::RagnarokAssetPluginGroup)
            .add_plugins(world::Plugin);
    }
}
