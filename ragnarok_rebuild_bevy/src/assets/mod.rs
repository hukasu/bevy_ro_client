pub mod grf;
pub mod rsm;
pub mod rsw;

use bevy::{
    app::{Plugin, PluginGroup as BevyPluginGroup, PluginGroupBuilder},
    asset::{AssetApp, AssetPlugin},
};

pub struct PluginGroup;

impl BevyPluginGroup for PluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AssetPlugin {
                file_path: "data/".to_owned(),
                ..Default::default()
            })
            .add_before::<AssetPlugin, RagnarokAssetReaderPlugin>(RagnarokAssetReaderPlugin)
            .add_after::<AssetPlugin, rsw::Plugin>(rsw::Plugin)
            .add_after::<AssetPlugin, rsm::Plugin>(rsm::Plugin)
    }
}

struct RagnarokAssetReaderPlugin;

impl Plugin for RagnarokAssetReaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_source(
            bevy::asset::io::AssetSourceId::Default,
            bevy::asset::io::AssetSourceBuilder::default().with_reader(|| {
                let grf = grf::AssetReader::new(&std::path::Path::new("data.grf")).unwrap();
                Box::new(grf)
            }),
        );
    }
}
