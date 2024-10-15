mod gnd_debug;
mod rsm_debug;
use bevy::app::{PluginGroup, PluginGroupBuilder};

pub struct DebugPlugin;

impl PluginGroup for DebugPlugin {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(gnd_debug::GndDebugPlugin)
            .add(rsm_debug::RsmDebugPlugin)
    }
}
