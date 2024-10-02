mod gnd;
mod rsm;
mod spr;

use bevy::app::PluginGroupBuilder;

pub use self::{
    gnd::GndMaterial,
    rsm::RsmMaterial,
    spr::{SprIndexedMaterial, SprTrueColorMaterial, SprUniform},
};

pub struct PluginGroup;

impl bevy::app::PluginGroup for PluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(gnd::Plugin)
            .add(rsm::Plugin)
            .add(spr::Plugin)
    }
}
