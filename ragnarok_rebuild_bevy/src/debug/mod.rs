mod show_rsm_normals;

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub struct DebugPlugin;

impl PluginGroup for DebugPlugin {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(show_rsm_normals::ShowRsmVertexNormalsPlugin)
    }
}
