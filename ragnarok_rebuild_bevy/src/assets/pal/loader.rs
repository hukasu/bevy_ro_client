use bevy::{asset::AsyncReadExt, prelude::Image};
use ragnarok_rebuild_assets::pal;

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Image;
    type Settings = ();
    type Error = pal::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let palette = pal::Palette::from_reader(&mut data.as_slice())?;

            Ok(super::palette_to_image(&palette))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pal"]
    }
}
