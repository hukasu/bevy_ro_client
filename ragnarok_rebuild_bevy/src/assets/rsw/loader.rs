use bevy::{
    asset::{io::Reader, AssetLoader as BevyAssetLoader, AsyncReadExt, Handle, LoadContext},
    audio::AudioSource,
    core::Name,
    ecs::world::World,
    hierarchy::BuildWorldChildren,
    math::{EulerRot, Quat, Vec3},
    pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle, PointLight, PointLightBundle},
    prelude::SpatialBundle,
    render::color::Color,
    scene::{Scene, SceneBundle},
    transform::{components::Transform, TransformBundle},
};
use ragnarok_rebuild_common::assets::rsw;

use crate::assets::paths;

pub struct AssetLoader;

impl AssetLoader {
    fn generate_world(rsw: &rsw::RSW, load_context: &mut LoadContext) -> Scene {
        bevy::log::trace!("Generating {:?} world.", load_context.path());

        let mut world = World::new();

        Self::set_ambient_light(rsw, &mut world, load_context);
        Self::spawn_directional_light(rsw, &mut world, load_context);
        Self::spawn_ground(rsw, &mut world, load_context);
        Self::spawn_animated_props(rsw, &mut world, load_context);
        Self::spawn_environmental_lights(rsw, &mut world, load_context);
        Self::spawn_environmental_sounds(rsw, &mut world, load_context);
        Self::spawn_world_water_plane(rsw, &mut world, load_context);

        Scene::new(world)
    }

    fn set_ambient_light(rsw: &rsw::RSW, world: &mut World, load_context: &mut LoadContext) {
        bevy::log::trace!("Setting ambient light of {:?}.", load_context.path());
        world.insert_resource(AmbientLight {
            color: Color::RgbaLinear {
                red: rsw.lighting_parameters.ambient_color[0],
                green: rsw.lighting_parameters.ambient_color[1],
                blue: rsw.lighting_parameters.ambient_color[2],
                alpha: 1.0,
            },
            brightness: 1.0,
        });
    }

    fn spawn_directional_light(rsw: &rsw::RSW, world: &mut World, load_context: &mut LoadContext) {
        bevy::log::trace!("Spawning directional light of {:?}", load_context.path());
        let base_distance = -2500.;
        let latitude_radians = (rsw.lighting_parameters.latitude as f32).to_radians();
        let longitude_radians = (rsw.lighting_parameters.longitude as f32).to_radians();

        let mut light_transform = Transform::from_xyz(0., base_distance, 0.);
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(longitude_radians));
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(latitude_radians));

        world.spawn((
            Name::new("DirectionalLight"),
            DirectionalLightBundle {
                directional_light: DirectionalLight {
                    color: Color::RgbaLinear {
                        red: rsw.lighting_parameters.diffuse_color[0],
                        green: rsw.lighting_parameters.diffuse_color[1],
                        blue: rsw.lighting_parameters.diffuse_color[2],
                        alpha: rsw.lighting_parameters.shadow_map_alpha,
                    },
                    illuminance: 10000.,
                    shadows_enabled: true,
                    ..Default::default()
                },
                transform: light_transform.looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            },
        ));
    }

    fn spawn_ground(rsw: &rsw::RSW, world: &mut World, load_context: &mut LoadContext) {
        bevy::log::trace!("Spawning ground of {:?}", load_context.path());

        let world_ground =
            load_context.load(format!("{}{}", paths::GROUND_FILES_FOLDER, rsw.gnd_file));

        world.spawn((
            Name::new("Ground"),
            super::components::Ground,
            SceneBundle {
                scene: world_ground,
                ..Default::default()
            },
        ));
    }

    fn spawn_animated_props(rsw: &rsw::RSW, world: &mut World, load_context: &mut LoadContext) {
        bevy::log::trace!("Spawning animated props of {:?}", load_context.path());
        world
            .spawn((
                Name::new("Models"),
                super::components::Models,
                SpatialBundle::default(),
            ))
            .with_children(|parent| {
                for prop in rsw.objects.0.iter() {
                    let prop_handle = load_context.load(format!(
                        "{}{}",
                        paths::MODEL_FILES_FOLDER,
                        prop.filename
                    ));
                    parent.spawn((
                        Name::new(prop.name.to_string()),
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
            });
    }

    fn spawn_environmental_lights(
        rsw: &rsw::RSW,
        world: &mut World,
        load_context: &mut LoadContext,
    ) {
        bevy::log::trace!("Spawning environmental lights of {:?}", load_context.path());
        world
            .spawn((
                Name::new("Lights"),
                super::components::EnvironmentalLights,
                SpatialBundle::default(),
            ))
            .with_children(|parent| {
                for light in rsw.objects.1.iter() {
                    parent.spawn((
                        Name::new(light.name.to_string()),
                        PointLightBundle {
                            transform: Transform::from_translation(Vec3::from_array(
                                light.position,
                            )),
                            point_light: PointLight {
                                color: Color::RgbaLinear {
                                    red: light.color[0],
                                    green: light.color[1],
                                    blue: light.color[2],
                                    alpha: 1.,
                                },
                                intensity: 32000.,
                                range: light.range,
                                shadows_enabled: true,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ));
                }
            });
    }

    fn spawn_environmental_sounds(
        rsw: &rsw::RSW,
        world: &mut World,
        load_context: &mut LoadContext,
    ) {
        bevy::log::trace!("Spawning environmental sounds of {:?}", load_context.path());
        world
            .spawn((
                Name::new("Sounds"),
                super::components::EnvironmentalSounds,
                SpatialBundle::default(),
            ))
            .with_children(|parent| {
                for sound in rsw.objects.2.iter() {
                    let audio_handle: Handle<AudioSource> =
                        load_context.load(format!("{}{}", paths::WAV_FILES_FOLDER, sound.filename));

                    parent.spawn((
                        Name::new(sound.name.to_string()),
                        TransformBundle {
                            local: Transform::from_translation(Vec3::from_array(sound.position)),
                            ..Default::default()
                        },
                        super::components::EnvironmentalSound {
                            source: audio_handle,
                            volume: sound.volume,
                        },
                    ));
                }
            });
    }

    fn spawn_world_water_plane(rsw: &rsw::RSW, world: &mut World, load_context: &mut LoadContext) {
        if let Some(water_plane_config) = &rsw.water_configuration {
            bevy::log::trace!("Spawning water plane of {:?}", load_context.path());
        } else {
            bevy::log::trace!("{:?} does not have a water plane.", load_context.path());
        }
    }
}

impl BevyAssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = ();
    type Error = super::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let rsw = rsw::RSW::from_reader(&mut data.as_slice())?;

            Ok(Self::generate_world(&rsw, load_context))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rsw"]
    }
}
