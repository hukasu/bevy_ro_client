mod rsm;

use bevy::app::PluginGroupBuilder;

pub use self::rsm::RsmMaterial;

pub struct PluginGroup;

impl bevy::app::PluginGroup for PluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(rsm::Plugin)
    }
}
