mod gnd;
mod water_plane;

use bevy::app::PluginGroupBuilder;

pub use self::{gnd::GndMaterial, water_plane::WaterPlaneMaterial};

pub struct PluginGroup;

impl bevy::app::PluginGroup for PluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(gnd::Plugin)
            .add(water_plane::Plugin)
    }
}
