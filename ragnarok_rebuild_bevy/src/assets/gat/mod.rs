mod assets;
mod components;
#[cfg(feature = "debug")]
mod debug;
mod ext;
mod loader;
mod resources;

use bevy::{
    app::Update,
    asset::AssetApp,
    ecs::{
        schedule::{
            common_conditions::{resource_exists, resource_removed},
            IntoSystemConfigs,
        },
        system::{Query, Res, ResMut},
    },
    math::{bounding::Aabb3d, Quat, Vec3},
    prelude::Triangle3d,
    render::primitives::Aabb,
};

use crate::{assets::rsw, helper::AabbExt, WorldTransform};

use self::ext::RaycastExt;
pub use self::resources::TileRayCast;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Assets
            .init_asset::<assets::Gat>()
            // AssetLoaders
            .register_asset_loader(loader::AssetLoader)
            // Resources
            .init_resource::<resources::HoveredTile>()
            // Register types
            .register_type::<components::Tiles>()
            .register_type::<resources::TileRayCast>()
            .register_type::<resources::HoveredTile>()
            // Systems
            .add_systems(
                Update,
                mouse_intersect_gat.run_if(resource_exists::<resources::TileRayCast>),
            )
            .add_systems(
                Update,
                clear_hovered_tile.run_if(resource_removed::<resources::TileRayCast>()),
            );

        #[cfg(feature = "debug")]
        {
            app.add_plugins(debug::Plugin);
        }
    }
}

fn mouse_intersect_gat(
    gats: Query<&components::Tiles>,
    rsws: Query<&rsw::World>,
    ray_cast: Res<TileRayCast>,
    mut hovered_tile: ResMut<resources::HoveredTile>,
    world_transform: Res<WorldTransform>,
) {
    let Ok(tiles) = gats.get_single() else {
        bevy::log::error!("There are none or more than one Gat spawned.");
        return;
    };
    let Ok(rsw) = rsws.get_single() else {
        bevy::log::error!("There are none or more than one Rsw spawned.");
        return;
    };

    let mut stack = vec![rsw::QuadTreeIndex::default()];
    let mut intersections = vec![];

    while let Some(head) = stack.pop() {
        // Bounding is in game space, and requires conversion to world space
        let bounding = Aabb3d::new(
            world_transform.transform_point(rsw.quad_tree[head].center.into()),
            world_transform
                .with_rotation(Quat::default())
                .transform_point(rsw.quad_tree[head].half_extents.into()),
        );

        if ray_cast.aabb_intersection_at(&bounding).is_some() {
            if head.is_leaf() {
                let node_tiles = tiles
                    .iter_tiles(
                        // QuadTree node is in game space, and requires conversion to world space
                        rsw.quad_tree[head].rotate(world_transform.with_rotation(Quat::default())),
                    )
                    .map(|tile_ref| {
                        let x = tile_ref.x as f32 - (tiles.width / 2) as f32;
                        let z = tile_ref.z as f32 - (tiles.height / 2) as f32;
                        let Some(tile_aabb) = Aabb::enclosing([
                            Vec3::new(
                                x,
                                tile_ref.tile.bottom_left * world_transform.scale.y,
                                z + 1.,
                            ),
                            Vec3::new(
                                x + 1.,
                                tile_ref.tile.bottom_right * world_transform.scale.y,
                                z + 1.,
                            ),
                            Vec3::new(x, tile_ref.tile.top_left * world_transform.scale.y, z),
                            Vec3::new(x + 1., tile_ref.tile.top_right * world_transform.scale.y, z),
                        ]) else {
                            unreachable!("Aabb is created from enclosing point.")
                        };
                        (
                            tile_ref,
                            tile_aabb.rotate(world_transform.with_scale(Vec3::splat(1.))),
                        )
                    })
                    .map(|(tile_ref, aabb)| (tile_ref, Aabb3d::new(aabb.center, aabb.half_extents)))
                    .flat_map(|(tile_ref, aabb)| {
                        ray_cast
                            .aabb_intersection_at(&aabb)
                            .map(|distance| (tile_ref, aabb, distance))
                    })
                    .filter(|(tile_ref, _, _)| {
                        let x = tile_ref.x as f32 - (tiles.width / 2) as f32;
                        let z = tile_ref.z as f32 - (tiles.height / 2) as f32;
                        ray_cast
                            .intersect_triangle(
                                Triangle3d::new(
                                    world_transform.with_scale(Vec3::splat(1.)).transform_point(
                                        Vec3::new(
                                            x,
                                            tile_ref.tile.bottom_left * world_transform.scale.y,
                                            z + 1.,
                                        ),
                                    ),
                                    world_transform.with_scale(Vec3::splat(1.)).transform_point(
                                        Vec3::new(
                                            x,
                                            tile_ref.tile.top_left * world_transform.scale.y,
                                            z,
                                        ),
                                    ),
                                    world_transform.with_scale(Vec3::splat(1.)).transform_point(
                                        Vec3::new(
                                            x + 1.,
                                            tile_ref.tile.bottom_right * world_transform.scale.y,
                                            z + 1.,
                                        ),
                                    ),
                                )
                                .reversed(),
                            )
                            .is_some()
                            || ray_cast
                                .intersect_triangle(
                                    Triangle3d::new(
                                        world_transform
                                            .with_scale(Vec3::splat(1.))
                                            .transform_point(Vec3::new(
                                                x + 1.,
                                                tile_ref.tile.bottom_right
                                                    * world_transform.scale.y,
                                                z + 1.,
                                            )),
                                        world_transform
                                            .with_scale(Vec3::splat(1.))
                                            .transform_point(Vec3::new(
                                                x,
                                                tile_ref.tile.top_left * world_transform.scale.y,
                                                z,
                                            )),
                                        world_transform
                                            .with_scale(Vec3::splat(1.))
                                            .transform_point(Vec3::new(
                                                x + 1.,
                                                tile_ref.tile.top_right * world_transform.scale.y,
                                                z,
                                            )),
                                    )
                                    .reversed(),
                                )
                                .is_some()
                    })
                    .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b));

                intersections.push(node_tiles);
            } else {
                let Some(top_right) = head.top_right() else {
                    unreachable!("Should have top right if it's not leaf")
                };
                stack.push(top_right);
                let Some(top_left) = head.top_left() else {
                    unreachable!("Should have top left if it's not leaf")
                };
                stack.push(top_left);
                let Some(bottom_right) = head.bottom_right() else {
                    unreachable!("Should have bottom right if it's not leaf")
                };
                stack.push(bottom_right);
                let Some(bottom_left) = head.bottom_left() else {
                    unreachable!("Should have bottom left if it's not leaf")
                };
                stack.push(bottom_left);
            }
        }
    }

    **hovered_tile = intersections
        .into_iter()
        .flatten()
        .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b))
        .map(|(tile, _, _)| tile);
}

fn clear_hovered_tile(mut hovered_tile: ResMut<resources::HoveredTile>) {
    **hovered_tile = None;
}
