mod asset;
mod loader;

use bevy::{
    app::{Plugin as BevyPlugin, Update},
    asset::{AssetApp, AssetEvent, Assets, Handle},
    audio::{AudioSourceBundle, PlaybackSettings, VolumeLevel},
    core::Name,
    ecs::{
        entity::Entity,
        event::EventReader,
        query::With,
        system::{Commands, Query, Res},
    },
    hierarchy::{BuildChildren, Parent},
    math::Vec3,
    pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle},
    render::color::Color,
    transform::{components::Transform, TransformBundle},
};

use crate::components::world;

pub use self::{asset::Asset, loader::AssetLoader};
pub use ragnarok_rebuild_common::assets::rsw::{error::Error, RSW};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_asset::<Asset>()
            .register_asset_loader(AssetLoader)
            .add_systems(Update, clear_loaded_asset)
            .add_systems(Update, set_ambient_light)
            .add_systems(Update, spawn_directional_light)
            .add_systems(Update, place_sounds);
    }
}

fn filter_events_that_are_tied_to_a_map(
    event: &AssetEvent<Asset>,
    query: &Query<(Entity, &Handle<Asset>), With<world::World>>,
) -> Option<(Entity, Handle<Asset>)> {
    if let AssetEvent::LoadedWithDependencies { id } = event {
        query
            .iter()
            .find(|query_item| &query_item.1.id() == id)
            .map(|(entity, handle)| (entity, handle.clone()))
    } else {
        None
    }
}

fn clear_loaded_asset(
    mut commands: Commands,
    query: Query<'_, '_, (Entity, &'_ Handle<Asset>), With<world::World>>,
    mut event_reader: EventReader<AssetEvent<Asset>>,
) {
    for (entity, _asset_handle) in event_reader
        .read()
        .filter_map(|event| filter_events_that_are_tied_to_a_map(event, &query))
    {
        bevy::log::debug!("Cleared Handle component.");
        commands.entity(entity).remove::<Handle<Asset>>();
    }
}

fn set_ambient_light(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Asset>), With<world::World>>,
    mut event_reader: EventReader<AssetEvent<Asset>>,
    rsw_assets: Res<Assets<Asset>>,
) {
    for (_, asset_handle) in event_reader
        .read()
        .filter_map(|event| filter_events_that_are_tied_to_a_map(event, &query))
    {
        if let Some(raw_rsw) = rsw_assets.get(asset_handle) {
            bevy::log::debug!("Set ambient light.");
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

fn spawn_directional_light(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Asset>), With<world::World>>,
    mut event_reader: EventReader<AssetEvent<Asset>>,
    rsw_assets: Res<Assets<Asset>>,
) {
    for (entity, asset_handle) in event_reader
        .read()
        .filter_map(|event| filter_events_that_are_tied_to_a_map(event, &query))
    {
        if let Some(raw_rsw) = rsw_assets.get(asset_handle) {
            bevy::log::debug!("Spawn directional light.");
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
            commands.entity(entity).add_child(directional_light);
        }
    }
}

fn place_sounds(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Asset>), With<world::World>>,
    world_sounds: Query<(Entity, &Parent), With<world::Sounds>>,
    mut event_reader: EventReader<AssetEvent<Asset>>,
    rsw_assets: Res<Assets<Asset>>,
) {
    for (entity, asset_handle) in event_reader
        .read()
        .filter_map(|event| filter_events_that_are_tied_to_a_map(event, &query))
    {
        if let Some(raw_rsw) = rsw_assets.get(asset_handle) {
            if let Some(world_sounds_entity) = world_sounds.iter().find_map(|world_sounds_entity| {
                if world_sounds_entity.1.get() == entity {
                    Some(world_sounds_entity.0)
                } else {
                    None
                }
            }) {
                let sounds = raw_rsw
                    .sound_handles
                    .iter()
                    .zip(raw_rsw.rsw.objects.2.iter())
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
                                        mode: bevy::audio::PlaybackMode::Loop,
                                        volume: bevy::audio::Volume::Relative(VolumeLevel::new(
                                            sound.volume,
                                        )),
                                        speed: 1.,
                                        spatial: true,
                                    },
                                },
                            ))
                            .id()
                    })
                    .collect::<Vec<_>>();
                commands.entity(world_sounds_entity).push_children(&sounds);
            } else {
                bevy::log::error!("Could not find child of World with Sounds component.");
            }
        }
    }
}
