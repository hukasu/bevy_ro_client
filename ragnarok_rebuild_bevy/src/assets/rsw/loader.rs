use bevy::asset::{io::Reader, AssetLoader as BevyAssetLoader, AsyncReadExt, LoadContext};

pub struct AssetLoader;

impl BevyAssetLoader for AssetLoader {
    type Asset = super::Asset;
    type Settings = ();
    type Error = super::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let rsw = super::RSW::from_reader(&mut data.as_slice())?;

            // TODO
            let ini_handle = None;
            let gnd_handle = None;
            let gat_handle = None;
            let source_handle = None;

            let rsm_handles = vec![];

            let sound_handles = rsw
                .objects
                .2
                .iter()
                .map(|sound| load_context.load(format!("data/wav/{}", sound.filename)))
                .collect();

            let effect_handles = vec![];

            Ok(Self::Asset {
                rsw,
                ini_handle,
                gnd_handle,
                gat_handle,
                source_handle,
                rsm_handles,
                sound_handles,
                effect_handles,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rsw"]
    }
}
