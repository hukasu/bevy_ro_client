use bevy::{
    asset::{io::Reader, AssetLoader as BevyAssetLoader, AsyncReadExt, LoadContext},
    render::texture::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
};

use crate::assets::paths;

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
            let gnd_handle =
                load_context.load(format!("{}{}", paths::GROUND_FILES_FOLDER, rsw.gnd_file));
            let gat_handle = None;
            let source_handle = None;

            let rsm_handles = rsw
                .objects
                .0
                .iter()
                .map(|model| {
                    load_context.load(format!("{}{}", paths::MODEL_FILES_FOLDER, model.filename))
                })
                .collect();

            let sound_handles = rsw
                .objects
                .2
                .iter()
                .map(|sound| {
                    load_context.load(format!("{}{}", paths::WAV_FILES_FOLDER, sound.filename))
                })
                .collect();

            let effect_handles = vec![];

            let water_textures = rsw.water_configuration.as_ref().map(|water_plane| {
                std::array::from_fn(|i| {
                    load_context.load_with_settings(
                        format!(
                            "{}water{}{:02}.jpg",
                            paths::WATER_TEXTURE_FILES_FOLDER,
                            water_plane.water_type,
                            i
                        ),
                        |m: &mut ImageLoaderSettings| {
                            m.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                                label: Some("WaterSample".to_owned()),
                                address_mode_u: ImageAddressMode::Repeat,
                                address_mode_v: ImageAddressMode::Repeat,
                                address_mode_w: ImageAddressMode::Repeat,
                                mag_filter: ImageFilterMode::Linear,
                                min_filter: ImageFilterMode::Linear,
                                ..Default::default()
                            })
                        },
                    )
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
