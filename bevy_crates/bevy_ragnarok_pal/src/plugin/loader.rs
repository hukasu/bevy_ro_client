use bevy_image::Image;
use ragnarok_pal::{Error, Pal};

use crate::pal_to_image;

pub struct AssetLoader;

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = Image;
    type Settings = ();
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy_asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let palette = Pal::from_reader(&mut data.as_slice())?;

        Ok(pal_to_image(palette))
    }

    fn extensions(&self) -> &[&str] {
        &["pal"]
    }
}
