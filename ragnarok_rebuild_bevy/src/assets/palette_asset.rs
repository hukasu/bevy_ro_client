use bevy::{
    asset::{Asset, AssetLoader, AsyncReadExt},
    reflect::TypePath,
};
use ragnarok_rebuild_common::assets::pal::{Palette, PaletteError};

#[derive(Debug, Asset, TypePath)]
pub struct PaletteAsset(Palette);

impl PaletteAsset {
    pub async fn load_palette<'a>(
        reader: &mut bevy::asset::io::Reader<'a>,
    ) -> Result<Self, PaletteError> {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;
        Ok(Self(Palette::from_bytes(&buffer)?))
    }
}

pub struct PaletteAssetLoader;

impl AssetLoader for PaletteAssetLoader {
    type Asset = PaletteAsset;
    type Settings = ();
    type Error = PaletteError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(PaletteAsset::load_palette(reader))
    }

    fn extensions(&self) -> &[&str] {
        &["pal"]
    }
}
