use bevy::{
    asset::{AsyncReadExt, LoadContext},
    core::Name,
    prelude::{SpatialBundle, World},
    scene::Scene,
};

use ragnarok_rebuild_assets::gat;

use super::components::{Tile, Tiles};

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = super::assets::Gat;
    type Settings = ();
    type Error = gat::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            bevy::log::trace!("Loading Gat {:?}.", load_context.path());

            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let gat = gat::Gat::from_reader(&mut data.as_slice())?;
            Self::generate_altitude(load_context, &gat);

            Ok(super::assets::Gat(gat))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gat"]
    }
}

impl AssetLoader {
    fn generate_altitude(load_context: &mut LoadContext, gat: &gat::Gat) {
        let mut world = World::new();

        world.spawn((
            Name::new("Tiles"),
            SpatialBundle::default(),
            Tiles {
                tiles: gat.tiles.iter().map(Tile::from).collect(),
                width: gat.width,
                height: gat.height,
            },
        ));

        load_context.add_labeled_asset("Scene".to_owned(), Scene::new(world));
    }
}
