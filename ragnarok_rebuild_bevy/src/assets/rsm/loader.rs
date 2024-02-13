use bevy::{
    asset::{io::Reader, AssetLoader as BevyAssetLoader, AsyncReadExt, LoadContext},
    utils::HashMap,
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
            bevy::log::trace!("Loading RSM {:?}.", load_context.path());
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let rsm = super::RSM::from_reader(&mut data.as_slice())?;

            let mut textures = HashMap::new();
            textures.insert(
                "root".into(),
                rsm.textures
                    .iter()
                    .map(|filename| {
                        load_context.load(format!("{}{}", paths::TEXTURES_FOLDER, filename))
                    })
                    .collect(),
            );
            for mesh in rsm.meshes.iter() {
                textures.insert(
                    mesh.name.clone(),
                    mesh.textures
                        .iter()
                        .map(|filename| {
                            load_context.load(format!("{}{}", paths::TEXTURES_FOLDER, filename))
                        })
                        .collect(),
                );
            }

            Ok(Self::Asset { rsm, textures })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rsm", "rsm2"]
    }
}
