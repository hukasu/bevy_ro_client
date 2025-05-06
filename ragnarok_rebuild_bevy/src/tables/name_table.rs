use std::{error::Error, fmt::Display};

use bevy::{
    asset::{Asset, AssetApp},
    platform::collections::HashSet,
    prelude::Deref,
    reflect::Reflect,
};

pub(super) struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Register Assets
            .init_asset::<NameTable>()
            // Register Asset Loaders
            .register_asset_loader(AssetLoader);
    }
}

#[derive(Debug, Asset, Reflect, Deref)]
pub struct NameTable {
    names: HashSet<String>,
}

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = NameTable;
    type Settings = ();
    type Error = NameTableError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let (decoded, _, has_replaced_caracters) = encoding_rs::EUC_KR.decode(&data);

        if has_replaced_caracters {
            Err(NameTableError::NotEucKr)
        } else {
            Ok(NameTable {
                names: HashSet::from_iter(
                    decoded
                        .lines()
                        .filter(|line| !line.starts_with("//"))
                        .filter(|line| !line.trim().is_empty())
                        .map(|line| line.trim().trim_end_matches("#").to_owned()),
                ),
            })
        }
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

#[derive(Debug)]
pub enum NameTableError {
    NotEucKr,
    Io(std::io::Error),
}

impl Display for NameTableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotEucKr => write!(f, "Name table file had an encoding that was not EUC_KR."),
            Self::Io(io) => write!(f, "An Io error occurred while reading name table. ({})", io),
        }
    }
}

impl Error for NameTableError {}

impl From<std::io::Error> for NameTableError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
