pub mod act;
pub mod gat;
pub mod gnd;
pub mod grf;
pub mod pal;
pub mod paths;
pub mod rsm;
pub mod rsw;
pub mod spr;
pub mod water_plane;

use bevy::app::{PluginGroup as BevyPluginGroup, PluginGroupBuilder};

pub struct PluginGroup;

impl BevyPluginGroup for PluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(act::Plugin)
            .add(gat::Plugin)
            .add(gnd::Plugin)
            .add(pal::Plugin)
            .add(rsm::Plugin)
            .add(rsw::Plugin)
            .add(spr::Plugin)
            .add(water_plane::Plugin)
    }
}
