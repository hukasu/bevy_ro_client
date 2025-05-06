use bevy::{
    asset::LoadContext,
    prelude::{Name, Transform, Visibility, World},
    scene::Scene,
};

use ragnarok_rebuild_assets::gat;

use super::components::{Tile, Tiles};

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = super::assets::Gat;
    type Settings = ();
    type Error = gat::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy::log::trace!("Loading Gat {:?}.", load_context.path());

        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let gat = gat::Gat::from_reader(&mut data.as_slice())?;
        Self::generate_altitude(load_context, &gat);

        Ok(super::assets::Gat(gat))
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
            Transform::default(),
            Visibility::default(),
            Tiles {
                tiles: gat.tiles.iter().map(Tile::from).collect(),
                width: gat.width,
                height: gat.height,
            },
        ));

        load_context.add_labeled_asset("Scene".to_owned(), Scene::new(world));
    }
}
