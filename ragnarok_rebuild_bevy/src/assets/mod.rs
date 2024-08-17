pub mod gnd;
pub mod grf;
pub mod paths;
pub mod rsm;
pub mod rsw;
pub mod water_plane;

use bevy::app::{PluginGroup as BevyPluginGroup, PluginGroupBuilder};

pub struct PluginGroup;

impl BevyPluginGroup for PluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(gnd::Plugin)
            .add(rsm::Plugin)
            .add(rsw::Plugin)
            .add(water_plane::Plugin)
    }
}
