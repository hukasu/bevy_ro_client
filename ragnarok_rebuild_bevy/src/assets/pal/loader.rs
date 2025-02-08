use bevy::prelude::Image;
use ragnarok_rebuild_assets::pal;

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Image;
    type Settings = ();
    type Error = pal::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let palette = pal::Pal::from_reader(&mut data.as_slice())?;

        Ok(super::palette_to_image(&palette))
    }

    fn extensions(&self) -> &[&str] {
        &["pal"]
    }
}
