pub mod actor_asset;
pub mod grf_asset_reader;
pub mod sprite_asset;

use bevy::{
    app::{Plugin, PluginGroup, PluginGroupBuilder},
    asset::{AssetApp, AssetPlugin},
};

use crate::assets::grf_asset_reader::GrfAssetReader;

use self::{
    actor_asset::{ActorAsset, ActorAssetLoader},
    sprite_asset::{SpriteAsset, SpriteAssetLoader},
};

pub struct RagnarokAssetPluginGroup;

impl PluginGroup for RagnarokAssetPluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<RagnarokAssetPluginGroup>()
            .add(AssetPlugin {
                file_path: "data/".to_owned(),
                ..Default::default()
            })
            .add_before::<AssetPlugin, RagnarokAssetReaderPlugin>(RagnarokAssetReaderPlugin)
            .add_after::<AssetPlugin, RagnarokAssetsPlugin>(RagnarokAssetsPlugin)
    }
}

struct RagnarokAssetReaderPlugin;

impl Plugin for RagnarokAssetReaderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_source(
            bevy::asset::io::AssetSourceId::Default,
            bevy::asset::io::AssetSourceBuilder::default().with_reader(|| {
                let grf = GrfAssetReader::new(&std::path::Path::new("data.grf")).unwrap();
                Box::new(grf)
            }),
        );
    }
}

struct RagnarokAssetsPlugin;

impl Plugin for RagnarokAssetsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<SpriteAsset>()
            .init_asset::<ActorAsset>()
            .register_asset_loader(SpriteAssetLoader)
            .register_asset_loader(ActorAssetLoader);
    }
}
