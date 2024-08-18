mod rsm_debug;

use bevy::app::{PluginGroup, PluginGroupBuilder};

pub struct DebugPlugin;

impl PluginGroup for DebugPlugin {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(rsm_debug::RsmDebugPlugin)
    }
}
