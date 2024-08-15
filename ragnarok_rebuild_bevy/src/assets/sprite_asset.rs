use bevy::{
    asset::{Asset, AssetLoader},
    reflect::TypePath,
};
use futures::AsyncReadExt;

use ragnarok_rebuild_common::assets::spr::{Sprite, SpriteError};

#[derive(Debug, Asset, TypePath)]
pub struct SpriteAsset {
    sprite: Sprite,
}

pub struct SpriteAssetLoader;

impl SpriteAssetLoader {
    async fn load_sprite<'a>(
        reader: &mut bevy::asset::io::Reader<'a>,
    ) -> Result<<Self as AssetLoader>::Asset, <Self as AssetLoader>::Error> {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;
        Sprite::from_bytes(&buffer).map(|sprite| SpriteAsset { sprite })
    }
}

impl AssetLoader for SpriteAssetLoader {
    type Asset = SpriteAsset;
    type Settings = ();
    type Error = SpriteError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(Self::load_sprite(reader))
    }

    fn extensions(&self) -> &[&str] {
        &["spr"]
    }
}
