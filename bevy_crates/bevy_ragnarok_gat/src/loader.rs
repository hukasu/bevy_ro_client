use bevy_asset::LoadContext;
use bevy_camera::{primitives::Aabb, visibility::Visibility};
use bevy_ecs::{hierarchy::ChildOf, name::Name, world::World};
use bevy_math::{FloatOrd, Vec3};
use bevy_ragnarok_quad_tree::TrackEntity;
use bevy_scene::Scene;
use bevy_transform::components::Transform;

use crate::Tile;

pub struct AssetLoader;

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = super::assets::Gat;
    type Settings = f32;
    type Error = ragnarok_gat::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy_log::trace!("Loading Gat {:?}.", load_context.path());

        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let gat = ragnarok_gat::Gat::from_reader(&mut data.as_slice())?;
        Self::generate_altitude(load_context, *settings, &gat);

        Ok(super::assets::Gat(gat))
    }

    fn extensions(&self) -> &[&str] {
        &["gat"]
    }
}

impl AssetLoader {
    fn generate_altitude(load_context: &mut LoadContext, tile_scale: f32, gat: &ragnarok_gat::Gat) {
        let mut world = World::new();

        let root = world
            .spawn((
                Name::new(
                    load_context
                        .path()
                        .file_stem()
                        .map(|osstr| osstr.to_string_lossy().into_owned())
                        .unwrap_or("Gat".to_owned()),
                ),
                Transform::default(),
                Visibility::default(),
            ))
            .id();

        let half_tile_scale = tile_scale / 2.;
        let scale = Vec3::new(tile_scale, 1., tile_scale);
        let offset = Vec3::new(gat.width as f32 / 2., 0., gat.height as f32 / 2.);
        for (i, tile) in gat.tiles.iter().enumerate() {
            let Ok(x) = u32::try_from(i).map(|i| i % gat.width) else {
                unreachable!("Should always fit in a u32.");
            };
            let Ok(z) = u32::try_from(i).map(|i| i / gat.width) else {
                unreachable!("Should always fit in a u32.");
            };

            let altitudes = [
                FloatOrd(tile.bottom_left_altitude()),
                FloatOrd(tile.bottom_right_altitude()),
                FloatOrd(tile.top_left_altitude()),
                FloatOrd(tile.top_right_altitude()),
            ];
            let Some(min) = altitudes.iter().min() else {
                unreachable!("Altitudes will never be empty.");
            };
            let Some(max) = altitudes.iter().max() else {
                unreachable!("Altitudes will never be empty.");
            };

            world.spawn((
                Name::new(format!("{x}/{z}")),
                Tile::from(tile),
                Transform::from_translation(
                    (Vec3::new(x as f32 + 0.5, max.0, z as f32 + 0.5) - offset) * scale,
                ),
                Aabb::from_min_max(
                    Vec3::new(-half_tile_scale, min.0 - max.0, -half_tile_scale),
                    Vec3::new(half_tile_scale, 0., half_tile_scale),
                ),
                ChildOf(root),
                TrackEntity,
            ));
        }

        load_context.add_labeled_asset("Scene".to_owned(), Scene::new(world));
    }
}
