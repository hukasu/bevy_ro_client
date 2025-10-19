pub mod paths;

use bevy::app::{PluginGroup as BevyPluginGroup, PluginGroupBuilder};

pub struct PluginGroup;

impl BevyPluginGroup for PluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
    }
}
