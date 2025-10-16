pub mod paths;
pub mod water_plane;

use bevy::app::{PluginGroup as BevyPluginGroup, PluginGroupBuilder};

pub struct PluginGroup;

impl BevyPluginGroup for PluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(water_plane::Plugin)
    }
}
