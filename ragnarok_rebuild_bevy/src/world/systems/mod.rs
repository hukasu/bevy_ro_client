use std::collections::BTreeMap;

use bevy::{
    asset::{AssetEvent, Assets, Handle},
    audio::{AudioSourceBundle, PlaybackMode, PlaybackSettings, Volume, VolumeLevel},
    core::Name,
    ecs::{
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    math::{Quat, Vec3},
    pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle, PbrBundle, StandardMaterial},
    render::{
        color::Color,
        mesh::{Indices, Mesh},
        render_resource::PrimitiveTopology,
    },
    transform::{components::Transform, TransformBundle},
};

use crate::{assets::rsw, water_plane};

use super::components;

#[derive(Debug, Event)]
pub struct RSWCompletedLoading {
    world: Entity,
    rsw: Handle<rsw::Asset>,
}

pub fn filter_events_that_are_tied_to_a_map(
    query: Query<(Entity, &Handle<rsw::Asset>), With<components::World>>,
    mut event_reader: EventReader<AssetEvent<rsw::Asset>>,
    mut event_writer: EventWriter<RSWCompletedLoading>,
) {
    event_writer.send_batch(
        event_reader
            .read()
            .filter_map(|event| {
                if let AssetEvent::LoadedWithDependencies { id } = event {
                    query
                        .iter()
                        .find(|query_item| &query_item.1.id() == id)
                        .map(|(entity, handle)| (entity, handle.clone()))
                } else {
                    None
                }
            })
            .map(|(world, rsw)| RSWCompletedLoading { world, rsw }),
    );
}

pub fn clear_loaded_asset(
    mut commands: Commands,
    mut event_reader: EventReader<RSWCompletedLoading>,
) {
    for RSWCompletedLoading {
        world: entity,
        rsw: _,
    } in event_reader.read()
    {
        bevy::log::trace!("Cleared Handle component.");
        commands.entity(*entity).remove::<Handle<rsw::Asset>>();
    }
}

pub fn set_ambient_light(
    mut commands: Commands,
    mut event_reader: EventReader<RSWCompletedLoading>,
    rsw_assets: Res<Assets<rsw::Asset>>,
) {
    for RSWCompletedLoading {
        world: _,
        rsw: asset_handle,
    } in event_reader.read()
    {
        if let Some(raw_rsw) = rsw_assets.get(asset_handle) {
            bevy::log::trace!("Set ambient light.");
            commands.insert_resource(AmbientLight {
                color: Color::RgbaLinear {
                    red: raw_rsw.rsw.lighting_parameters.ambient_red,
                    green: raw_rsw.rsw.lighting_parameters.ambient_green,
                    blue: raw_rsw.rsw.lighting_parameters.ambient_blue,
                    alpha: 1.,
                },
                brightness: raw_rsw.rsw.lighting_parameters.shadow_map_alpha,
            });
        }
    }
}

pub fn spawn_directional_light(
    mut commands: Commands,
    mut event_reader: EventReader<RSWCompletedLoading>,
    rsw_assets: Res<Assets<rsw::Asset>>,
) {
    for RSWCompletedLoading {
        world: entity,
        rsw: asset_handle,
    } in event_reader.read()
    {
        if let Some(raw_rsw) = rsw_assets.get(asset_handle) {
            bevy::log::trace!("Spawn directional light.");
            let base_distance = 1000.;
            let latitude_radians = (raw_rsw.rsw.lighting_parameters.latitude as f32).to_radians();
            let longitude_radians = (raw_rsw.rsw.lighting_parameters.longitude as f32).to_radians();
            let spherical_coordinate = Transform::from_xyz(
                base_distance * longitude_radians.sin() * latitude_radians.cos(),
                base_distance * longitude_radians.cos(),
                base_distance * longitude_radians.sin() * latitude_radians.sin(),
            );

            let directional_light = commands
                .spawn(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        color: Color::RgbaLinear {
                            red: raw_rsw.rsw.lighting_parameters.diffuse_red,
                            green: raw_rsw.rsw.lighting_parameters.diffuse_green,
                            blue: raw_rsw.rsw.lighting_parameters.diffuse_blue,
                            alpha: 1.,
                        },
                        illuminance: 32000.,
                        shadows_enabled: true,
                        ..Default::default()
                    },
                    transform: spherical_coordinate.looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                })
                .id();
            commands.entity(*entity).add_child(directional_light);
        }
    }
}

pub fn place_sounds(
    mut commands: Commands,
    mut event_reader: EventReader<RSWCompletedLoading>,
    rsw_assets: Res<Assets<rsw::Asset>>,
) {
    for RSWCompletedLoading {
        world: entity,
        rsw: asset_handle,
    } in event_reader.read()
    {
        if let Some(rsw_asset) = rsw_assets.get(asset_handle) {
            let world_sounds = commands
                .spawn((
                    components::Sounds,
                    Name::new("Sounds"),
                    TransformBundle::default(),
                ))
                .id();
            commands.entity(*entity).add_child(world_sounds);

            let sounds = rsw_asset
                .sound_handles
                .iter()
                .zip(rsw_asset.rsw.objects.2.iter())
                .map(|(handle, sound)| {
                    commands
                        .spawn((
                            Name::new(sound.name.to_string()),
                            TransformBundle {
                                local: Transform::from_xyz(
                                    sound.position.0,
                                    sound.position.1,
                                    sound.position.2,
                                ),
                                ..Default::default()
                            },
                            AudioSourceBundle {
                                source: handle.clone(),
                                settings: PlaybackSettings {
                                    paused: false,
                                    mode: PlaybackMode::Loop,
                                    volume: Volume::Relative(VolumeLevel::new(sound.volume)),
                                    speed: 1.,
                                    spatial: true,
                                },
                            },
                        ))
                        .id()
                })
                .collect::<Vec<_>>();
            commands.entity(world_sounds).push_children(&sounds);
        }
    }
}

pub fn spawn_water_plane(
    mut commands: Commands,
    mut event_reader: EventReader<RSWCompletedLoading>,
    rsw_assets: Res<Assets<rsw::Asset>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    for RSWCompletedLoading {
        world: entity,
        rsw: asset_handle,
    } in event_reader.read()
    {
        if let Some(rsw_asset) = rsw_assets.get(asset_handle) {
            let Some(water_configuration) = &rsw_asset.rsw.water_configuration else {
                continue;
            };
            let Some(water_textures) = &rsw_asset.water_textures else {
                bevy::log::error!("World has water plane but did not have water textures.");
                continue;
            };
            use ragnarok_rebuild_common::assets::rsw::Range;

            let find_vertex = |vertexes: &[[f32; 3]], range: &[f32; 2]| -> Option<u32> {
                vertexes
                    .iter()
                    .position(|vertex: &[f32; 3]| {
                        (vertex[0] - range[0]).abs() < std::f32::EPSILON
                            && (vertex[2] - range[1]).abs() < std::f32::EPSILON
                    })
                    .map(|pos| pos as u32)
            };

            // Get the root and one of the leaves from the QuadTree
            let quad_tree_root = &rsw_asset.rsw.quad_tree.ranges[0];
            let smallest_quad = &rsw_asset.rsw.quad_tree.ranges
                [ragnarok_rebuild_common::assets::rsw::QUAD_TREE_MAX_DEPTH];

            // Get all leaves that contains the water level
            let quad_tree_ranges_that_contains_water = rsw_asset
                .rsw
                .quad_tree
                .ranges
                .iter()
                .filter(|range| {
                    (range.radius.0 - smallest_quad.radius.0).abs() < std::f32::EPSILON
                        && range.top.1 > water_configuration.water_level
                })
                .map(|range| {
                    let x =
                        (range.bottom.0 - quad_tree_root.bottom.0) / (smallest_quad.radius.0 * 2.);
                    let z =
                        (range.bottom.2 - quad_tree_root.bottom.2) / (smallest_quad.radius.2 * 2.);
                    ((x.round() as usize, z.round() as usize), range)
                })
                .collect::<BTreeMap<(usize, usize), &Range>>();

            if quad_tree_ranges_that_contains_water.is_empty() {
                bevy::log::trace!("World has empty water plane.");
                continue;
            }

            let mut vertexes = vec![];
            let mut uvs = vec![];
            let mut indeces = vec![];
            for (key, range) in quad_tree_ranges_that_contains_water.iter() {
                // TODO verify if there is ever water on the edges and treat that case
                // The bottom left vertex might already be included if the tile to
                // the left, down, or diagonally down-left also contains water
                let bottom_left = if [
                    (key.0 - 1, key.1),
                    (key.0, key.1 - 1),
                    (key.0 - 1, key.1 - 1),
                ]
                .iter()
                .any(|adj_key| quad_tree_ranges_that_contains_water.contains_key(adj_key))
                {
                    let Some(pos) = find_vertex(&vertexes, &[range.bottom.0, range.bottom.2])
                    else {
                        bevy::log::error!(
                            "Failed to build water plane. Could not find index of bottom left vertex."
                        );
                        continue;
                    };
                    pos
                } else {
                    let bottom_left = vertexes.len();
                    vertexes.push([range.bottom.0, 0., range.bottom.2]);
                    uvs.push([key.0 as f32, key.1 as f32]);
                    bottom_left as u32
                };
                // The bottom right vertex might already be included if the tile below also contains water
                let bottom_right = if quad_tree_ranges_that_contains_water
                    .contains_key(&(key.0, key.1 - 1))
                {
                    let Some(pos) = find_vertex(&vertexes, &[range.top.0, range.bottom.2]) else {
                        bevy::log::error!(
                                "Failed to build water plane. Could not find index of bottom right vertex."
                            );
                        continue;
                    };
                    pos
                } else {
                    let bottom_right = vertexes.len();
                    vertexes.push([range.top.0, 0., range.bottom.2]);
                    uvs.push([(key.0 + 1) as f32, key.1 as f32]);
                    bottom_right as u32
                };
                // The top left vertex might already be included if the tile below also contains water
                let top_left = if quad_tree_ranges_that_contains_water
                    .contains_key(&(key.0 - 1, key.1))
                {
                    let Some(pos) = find_vertex(&vertexes, &[range.bottom.0, range.top.2]) else {
                        bevy::log::error!(
                            "Failed to build water plane. Could not find index of top left vertex."
                        );
                        continue;
                    };
                    pos
                } else {
                    let top_left = vertexes.len();
                    vertexes.push([range.bottom.0, 0., range.top.2]);
                    uvs.push([key.0 as f32, (key.1 + 1) as f32]);
                    top_left as u32
                };
                // The top right vertex is always added
                let top_right = vertexes.len() as u32;
                vertexes.push([range.top.0, 0., range.top.2]);
                uvs.push([(key.0 + 1) as f32, (key.1 + 1) as f32]);
                // bevy::render::mesh::shape::Box;
                indeces.append(&mut vec![
                    bottom_left,
                    bottom_right,
                    top_left,
                    bottom_right,
                    top_right,
                    top_left,
                ]);
            }
            let normals = vec![[0., 1., 0.]; vertexes.len()];

            let mesh = mesh_assets.add(
                Mesh::new(PrimitiveTopology::TriangleList)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertexes)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                    .with_indices(Some(Indices::U32(indeces))),
            );

            let water_plane_entity = commands
                .spawn((
                    water_plane::WaterPlane::new(
                        water_textures.clone(),
                        water_configuration.texture_cyclical_interval,
                    ),
                    Name::new("WaterPlane"),
                    PbrBundle {
                        mesh,
                        transform: Transform::from_xyz(
                            quad_tree_root.center.0,
                            water_configuration.water_level,
                            quad_tree_root.center.2,
                        ),
                        material: material_assets.add(StandardMaterial {
                            base_color: Color::WHITE.with_a(0.56),
                            base_color_texture: Some(water_textures[0].clone()),
                            alpha_mode: bevy::pbr::AlphaMode::Blend,
                            ior: 1.33,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                ))
                .id();
            commands.entity(*entity).add_child(water_plane_entity);
        }
    }
}

pub fn spawn_models(
    mut commands: Commands,
    mut event_reader: EventReader<RSWCompletedLoading>,
    rsw_assets: Res<Assets<rsw::Asset>>,
) {
    for RSWCompletedLoading {
        world: entity,
        rsw: asset_handle,
    } in event_reader.read()
    {
        if let Some(rsw_asset) = rsw_assets.get(asset_handle) {
            let world_models = commands
                .spawn((
                    components::Models,
                    Name::new("Models"),
                    TransformBundle::default(),
                ))
                .id();
            commands.entity(*entity).add_child(world_models);

            let models = rsw_asset
                .rsw
                .objects
                .0
                .iter()
                .zip(rsw_asset.rsm_handles.iter())
                .map(|(rsm, rsm_handle)| {
                    commands
                        .spawn((
                            Name::new(rsm.name.to_string()),
                            rsm_handle.clone(),
                            TransformBundle {
                                local: Transform {
                                    translation: Vec3::from_array(rsm.position.into()),
                                    rotation: Quat::from_euler(
                                        bevy::math::EulerRot::XYZ,
                                        rsm.rotation.0,
                                        rsm.rotation.1,
                                        rsm.rotation.2,
                                    ),
                                    scale: Vec3::from_array(rsm.scale.into()),
                                },
                                ..Default::default()
                            },
                        ))
                        .id()
                })
                .collect::<Box<[_]>>();
            commands.entity(world_models).push_children(&models);
        }
    }
}

pub fn spawn_plane(
    mut commands: Commands,
    mut event_reader: EventReader<RSWCompletedLoading>,
    rsw_assets: Res<Assets<rsw::Asset>>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
) {
    for RSWCompletedLoading {
        world: entity,
        rsw: asset_handle,
    } in event_reader.read()
    {
        if let Some(rsw_asset) = rsw_assets.get(asset_handle) {
            let quad_tree_root = &rsw_asset.rsw.quad_tree.ranges[0];
            let plane = commands
                .spawn((
                    Name::new("WorldBottom"),
                    PbrBundle {
                        mesh: mesh_assets.add(bevy::render::mesh::shape::Plane::default().into()),
                        transform: Transform::from_xyz(
                            quad_tree_root.center.0,
                            quad_tree_root.center.1,
                            quad_tree_root.center.2,
                        )
                        .with_scale(Vec3::new(
                            -quad_tree_root.radius.0 * 2.,
                            1.,
                            quad_tree_root.radius.2 * 2.,
                        )),
                        ..Default::default()
                    },
                ))
                .id();
            commands.entity(*entity).add_child(plane);
        }
    }
}
