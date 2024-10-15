use std::time::Duration;

use bevy::{
    asset::{io::Reader, AsyncReadExt, Handle, LoadContext},
    audio::AudioSource,
    color::{Color, Luminance},
    core::Name,
    hierarchy::BuildWorldChildren,
    math::{EulerRot, Quat, Vec3},
    pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle, PointLight, PointLightBundle},
    prelude::{Entity, SpatialBundle, TransformBundle},
    render::primitives::Aabb,
    scene::{Scene, SceneBundle},
    time::Timer,
    transform::components::Transform,
};
use ragnarok_rebuild_assets::rsw;

use crate::assets::{
    gnd, paths,
    rsw::{
        components::{AnimatedProp, DiffuseLight, EnvironmentalLight, EnvironmentalSound, World},
        EnvironmentalEffect,
    },
};

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = ();
    type Error = super::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let rsw = rsw::Rsw::from_reader(&mut data.as_slice())?;

            Ok(Self::generate_world(&rsw, load_context))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rsw"]
    }
}

impl AssetLoader {
    const UNNAMED_RSW: &str = "Unnamed Rsw";
    const DIRECTIONAL_LIGHT_LUX: f32 = 4300.;

    fn generate_world(rsw: &rsw::Rsw, load_context: &mut LoadContext) -> Scene {
        bevy::log::trace!("Generating {:?} world.", load_context.path());

        let mut world = bevy::ecs::world::World::new();

        Self::set_ambient_light(rsw, &mut world, load_context);
        let directional_light = Self::spawn_directional_light(rsw, &mut world, load_context);
        let ground = Self::spawn_ground(rsw, &mut world, load_context);
        let tiles = Self::spawn_tiles(rsw, &mut world, load_context);
        let animated_props = Self::spawn_animated_props(rsw, &mut world, load_context);
        let environmental_lights = Self::spawn_environmental_lights(rsw, &mut world, load_context);
        let environmental_sounds = Self::spawn_environmental_sounds(rsw, &mut world, load_context);
        let environmental_effects =
            Self::spawn_environmental_effects(rsw, &mut world, load_context);

        let filename = match load_context.path().file_name() {
            Some(filename) => filename.to_str().unwrap_or(Self::UNNAMED_RSW),
            None => Self::UNNAMED_RSW,
        };

        world
            .spawn((
                Name::new(filename.to_string()),
                SpatialBundle::default(),
                World {
                    has_lights: !rsw.lights.is_empty(),
                    has_sounds: !rsw.sounds.is_empty(),
                    has_effects: !rsw.effects.is_empty(),
                    quad_tree: super::QuadTree::from(&rsw.quad_tree),
                },
            ))
            .push_children(&[
                directional_light,
                ground,
                tiles,
                animated_props,
                environmental_lights,
                environmental_sounds,
                environmental_effects,
            ]);

        Scene::new(world)
    }

    fn set_ambient_light(
        rsw: &rsw::Rsw,
        world: &mut bevy::prelude::World,
        load_context: &mut LoadContext,
    ) {
        bevy::log::trace!("Setting ambient light of {:?}.", load_context.path());
        world.insert_resource(AmbientLight {
            color: Color::linear_rgb(
                rsw.lighting_parameters.ambient_color[0],
                rsw.lighting_parameters.ambient_color[1],
                rsw.lighting_parameters.ambient_color[2],
            ),
            ..Default::default()
        });
    }

    fn spawn_directional_light(
        rsw: &rsw::Rsw,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning directional light of {:?}", load_context.path());
        let base_distance = -500.;
        let latitude_radians = (rsw.lighting_parameters.latitude as f32).to_radians();
        let longitude_radians = (rsw.lighting_parameters.longitude as f32).to_radians();

        let mut light_transform = Transform::from_xyz(0., base_distance, 0.);
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(longitude_radians));
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(latitude_radians));

        let directional_light_color = Color::linear_rgb(
            rsw.lighting_parameters.diffuse_color[0],
            rsw.lighting_parameters.diffuse_color[1],
            rsw.lighting_parameters.diffuse_color[2],
        );
        world
            .spawn((
                Name::new("DirectionalLight"),
                DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        color: directional_light_color,
                        illuminance: Self::DIRECTIONAL_LIGHT_LUX
                            * directional_light_color.luminance(),
                        shadows_enabled: true,
                        ..Default::default()
                    },
                    transform: light_transform.looking_at(Vec3::ZERO, Vec3::Y),
                    ..Default::default()
                },
                DiffuseLight,
            ))
            .id()
    }

    fn spawn_ground(
        rsw: &rsw::Rsw,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning ground of {:?}", load_context.path());

        let rsw_water_plane = rsw.water_configuration;
        let world_ground = load_context
            .loader()
            .with_settings(move |settings: &mut gnd::AssetLoaderSettings| {
                settings.water_plane = rsw_water_plane;
            })
            .load(format!("{}{}", paths::GROUND_FILES_FOLDER, rsw.gnd_file));

        world
            .spawn((
                Name::new(rsw.gnd_file.to_string()),
                SceneBundle {
                    scene: world_ground,
                    ..Default::default()
                },
            ))
            .id()
    }

    fn spawn_tiles(
        rsw: &rsw::Rsw,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning tiles of {:?}", load_context.path());

        let world_tiles = load_context.load(format!(
            "{}{}#Scene",
            paths::ALTITUDE_FILES_FOLDER,
            rsw.gat_file
        ));

        world
            .spawn((
                Name::new(rsw.gat_file.to_string()),
                SceneBundle {
                    scene: world_tiles,
                    ..Default::default()
                },
            ))
            .id()
    }

    fn spawn_animated_props(
        rsw: &rsw::Rsw,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning animated props of {:?}", load_context.path());
        world
            .spawn((Name::new("Models"), SpatialBundle::default()))
            .with_children(|parent| {
                for prop in rsw.models.iter() {
                    let prop_handle = load_context.load(format!(
                        "{}{}",
                        paths::MODEL_FILES_FOLDER,
                        prop.filename
                    ));
                    parent.spawn((
                        Name::new(prop.name.to_string()),
                        AnimatedProp {
                            animation_type: prop.animation_type,
                            animation_speed: prop.animation_speed,
                        },
                        SceneBundle {
                            scene: prop_handle,
                            transform: Transform {
                                translation: Vec3::from_array(prop.position),
                                rotation: Quat::from_euler(
                                    EulerRot::ZXY,
                                    prop.rotation[2].to_radians(),
                                    prop.rotation[0].to_radians(),
                                    prop.rotation[1].to_radians(),
                                ),
                                scale: Vec3::from_array(prop.scale),
                            },
                            ..Default::default()
                        },
                    ));
                }
            })
            .id()
    }

    fn spawn_environmental_lights(
        rsw: &rsw::Rsw,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning environmental lights of {:?}", load_context.path());
        world
            .spawn((Name::new("Lights"), SpatialBundle::default()))
            .with_children(|parent| {
                for light in rsw.lights.iter() {
                    parent.spawn((
                        Name::new(light.name.to_string()),
                        PointLightBundle {
                            transform: Transform::from_translation(Vec3::from_array(
                                light.position,
                            )),
                            point_light: PointLight {
                                color: Color::linear_rgb(
                                    light.color[0],
                                    light.color[1],
                                    light.color[2],
                                ),
                                range: light.range / 5.,
                                shadows_enabled: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        Aabb::from_min_max(
                            -Vec3::splat(light.range / 2.),
                            Vec3::splat(light.range / 2.),
                        ),
                        EnvironmentalLight { range: light.range },
                    ));
                }
            })
            .id()
    }

    fn spawn_environmental_sounds(
        rsw: &rsw::Rsw,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning environmental sounds of {:?}", load_context.path());
        world
            .spawn((Name::new("Sounds"), SpatialBundle::default()))
            .with_children(|parent| {
                for sound in rsw.sounds.iter() {
                    let audio_handle: Handle<AudioSource> =
                        load_context.load(format!("{}{}", paths::WAV_FILES_FOLDER, sound.filename));

                    parent.spawn((
                        Name::new(sound.name.to_string()),
                        TransformBundle {
                            local: Transform::from_translation(Vec3::from_array(sound.position)),
                            ..Default::default()
                        },
                        EnvironmentalSound {
                            name: sound.name.to_string(),
                            source: audio_handle,
                            position: Transform::from_translation(Vec3::from_slice(
                                &sound.position,
                            )),
                            volume: sound.volume,
                            range: sound.range,
                            cycle: Timer::new(
                                Duration::from_secs_f32(if sound.cycle < f32::EPSILON {
                                    bevy::log::warn!("{} had cycle set to 0. seconds. Changing to default 4. seconds.", sound.name);
                                    4.
                                } else {
                                    sound.cycle
                                }),
                                bevy::time::TimerMode::Repeating,
                            ),
                        },
                    ));
                }
            })
            .id()
    }

    fn spawn_environmental_effects(
        rsw: &rsw::Rsw,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!(
            "Spawning environmental effects of {:?}",
            load_context.path()
        );
        world
            .spawn((Name::new("Effects"), SpatialBundle::default()))
            .with_children(|parent| {
                for effect in rsw.effects.iter() {
                    parent.spawn((
                        Name::new(effect.name.to_string()),
                        TransformBundle {
                            local: Transform::from_translation(Vec3::from_array(effect.position)),
                            ..Default::default()
                        },
                        EnvironmentalEffect,
                    ));
                }
            })
            .id()
    }
}
