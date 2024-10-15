mod assets;
mod components;
#[cfg(feature = "debug")]
mod debug;
mod loader;
mod resources;

use bevy::{
    app::Update,
    asset::AssetApp,
    math::{
        bounding::{Aabb3d, RayCast3d},
        Quat, Vec3,
    },
    prelude::{Camera, GlobalTransform, IntoSystemConfigs, Query, Res, ResMut, With},
    render::primitives::Aabb,
    window::{PrimaryWindow, Window},
};

use crate::{assets::rsw, helper::AabbExt, WorldTransform};

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
            .register_type::<resources::HoveredTile>()
            // Systems
            .add_systems(Update, mouse_intersect_gat.run_if(is_mouse_free));

        #[cfg(feature = "debug")]
        {
            app.add_plugins(debug::Plugin);
        }
    }
}

fn is_mouse_free(windows: Query<&Window, With<PrimaryWindow>>) -> bool {
    if let Ok(primary_window) = windows.get_single() {
        matches!(
            primary_window.cursor.grab_mode,
            bevy::window::CursorGrabMode::None
        )
    } else {
        true
    }
}

fn mouse_intersect_gat(
    gats: Query<&components::Tiles>,
    rsws: Query<&rsw::World>,
    mut hovered_tile: ResMut<resources::HoveredTile>,
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    world_transform: Res<WorldTransform>,
) {
    let Ok(primary_window) = primary_windows.get_single() else {
        bevy::log::error!("There are none or more than one PrimaryWindow spawned.");
        return;
    };

    let Ok((camera, camera_transform)) = cameras.get_single() else {
        bevy::log::error!("There are none or more than one PrimCameraaryWindow spawned.");
        return;
    };

    let Ok(tiles) = gats.get_single() else {
        bevy::log::error!("There are none or more than one Gat spawned.");
        return;
    };
    let Ok(rsw) = rsws.get_single() else {
        bevy::log::error!("There are none or more than one Rsw spawned.");
        return;
    };

    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Ray is in world space
    let ray_cast = RayCast3d::from_ray(ray, 100.);

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
                #[allow(clippy::expect_used)]
                let node_tiles = tiles
                    .iter_tiles(
                        // QuadTree node is in game space, and requires conversion to world space
                        rsw.quad_tree[head].rotate(world_transform.with_rotation(Quat::default())),
                    )
                    .map(|tile_ref| {
                        let x = tile_ref.x as f32 - (tiles.width / 2) as f32;
                        let z = tile_ref.z as f32 - (tiles.height / 2) as f32;
                        (
                            tile_ref,
                            Aabb::enclosing([
                                Vec3::new(
                                    x,
                                    tile_ref.tile.bottom_left * world_transform.scale.y,
                                    z,
                                ),
                                Vec3::new(
                                    x + 1.,
                                    tile_ref.tile.bottom_right * world_transform.scale.y,
                                    z,
                                ),
                                Vec3::new(
                                    x,
                                    tile_ref.tile.top_left * world_transform.scale.y,
                                    z + 1.,
                                ),
                                Vec3::new(
                                    x + 1.,
                                    tile_ref.tile.top_right * world_transform.scale.y,
                                    z + 1.,
                                ),
                            ])
                            .expect("Aabb is created from enclosing point.")
                            .rotate(world_transform.with_scale(Vec3::splat(1.))),
                        )
                    })
                    .map(|(tile_ref, aabb)| (tile_ref, Aabb3d::new(aabb.center, aabb.half_extents)))
                    .flat_map(|(tile_ref, aabb)| {
                        ray_cast
                            .aabb_intersection_at(&aabb)
                            .map(|distance| (tile_ref, aabb, distance))
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
