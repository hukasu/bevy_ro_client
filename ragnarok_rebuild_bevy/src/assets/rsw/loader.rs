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

            let rsm_handles = rsw
                .objects
                .0
                .iter()
                .map(|model| load_context.load(format!("data/model/{}", model.filename)))
                .collect();

            let sound_handles = rsw
                .objects
                .2
                .iter()
                .map(|sound| load_context.load(format!("data/wav/{}", sound.filename)))
                .collect();

            let effect_handles = vec![];

            let water_textures = rsw.water_configuration.as_ref().map(|water_plane| {
                std::array::from_fn(|i| {
                    load_context.load(format!(
                        "data/texture/워터/water{}{:02}.jpg",
                        water_plane.water_type, i
                    ))
                })
            });

            Ok(Self::Asset {
                rsw,
                ini_handle,
                gnd_handle,
                gat_handle,
                source_handle,
                rsm_handles,
                sound_handles,
                effect_handles,
                water_textures,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rsw"]
    }
}
