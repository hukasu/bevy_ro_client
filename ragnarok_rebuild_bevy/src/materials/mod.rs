mod gnd;
mod rsm;
mod spr;
mod water_plane;

use bevy::app::PluginGroupBuilder;

pub use self::{
    gnd::GndMaterial,
    rsm::RsmMaterial,
    spr::{SprIndexedMaterial, SprTrueColorMaterial, SprUniform},
    water_plane::WaterPlaneMaterial,
};

pub struct PluginGroup;

impl bevy::app::PluginGroup for PluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(gnd::Plugin)
            .add(rsm::Plugin)
            .add(spr::Plugin)
            .add(water_plane::Plugin)
    }
}
