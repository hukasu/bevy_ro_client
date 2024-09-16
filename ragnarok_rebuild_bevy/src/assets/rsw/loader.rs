use bevy::{
    asset::{io::Reader, AsyncReadExt, Handle, LoadContext},
    audio::AudioSource,
    color::Color,
    core::Name,
    hierarchy::BuildWorldChildren,
    math::{EulerRot, Quat, Vec3},
    pbr::{AmbientLight, DirectionalLight, DirectionalLightBundle, PointLight, PointLightBundle},
    prelude::{Entity, SpatialBundle, TransformBundle},
    scene::{Scene, SceneBundle},
    transform::components::Transform,
};
use ragnarok_rebuild_assets::rsw;
use serde::{Deserialize, Serialize};

use crate::assets::{
    paths,
    rsw::{
        components::{AnimatedProp, DiffuseLight, EnvironmentalLight, EnvironmentalSound, World},
        EnvironmentalEffect,
    },
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RswSettings {
    pub is_indoor: bool,
}

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = RswSettings;
    type Error = super::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let rsw = rsw::RSW::from_reader(&mut data.as_slice())?;

            Ok(Self::generate_world(&rsw, settings, load_context))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rsw"]
    }
}

impl AssetLoader {
    const UNNAMED_RSW: &str = "Unnamed Rsw";

    fn generate_world(
        rsw: &rsw::RSW,
        settings: &RswSettings,
        load_context: &mut LoadContext,
    ) -> Scene {
        bevy::log::trace!("Generating {:?} world.", load_context.path());

        let mut world = bevy::ecs::world::World::new();

        Self::set_ambient_light(rsw, &mut world, load_context);
        let directional_light =
            Self::spawn_directional_light(rsw, &mut world, settings, load_context);
        let ground = Self::spawn_ground(rsw, &mut world, load_context);
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
                World,
            ))
            .push_children(&[
                directional_light,
                ground,
                animated_props,
                environmental_lights,
                environmental_sounds,
                environmental_effects,
            ]);

        Scene::new(world)
    }

    fn set_ambient_light(
        rsw: &rsw::RSW,
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
        rsw: &rsw::RSW,
        world: &mut bevy::ecs::world::World,
        settings: &RswSettings,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning directional light of {:?}", load_context.path());
        let base_distance = -500.;
        let latitude_radians = (rsw.lighting_parameters.latitude as f32).to_radians();
        let longitude_radians = (rsw.lighting_parameters.longitude as f32).to_radians();

        let mut light_transform = Transform::from_xyz(0., base_distance, 0.);
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(longitude_radians));
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(latitude_radians));

        world
            .spawn((
                Name::new("DirectionalLight"),
                DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        color: Color::linear_rgb(
                            rsw.lighting_parameters.diffuse_color[0],
                            rsw.lighting_parameters.diffuse_color[1],
                            rsw.lighting_parameters.diffuse_color[2],
                        ),
                        illuminance: if settings.is_indoor { 100. } else { 10000. },
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
        rsw: &rsw::RSW,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning ground of {:?}", load_context.path());

        let world_ground =
            load_context.load(format!("{}{}", paths::GROUND_FILES_FOLDER, rsw.gnd_file));

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

    fn spawn_animated_props(
        rsw: &rsw::RSW,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning animated props of {:?}", load_context.path());
        world
            .spawn((Name::new("Models"), SpatialBundle::default()))
            .with_children(|parent| {
                for prop in rsw.objects.0.iter() {
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
        rsw: &rsw::RSW,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning environmental lights of {:?}", load_context.path());
        world
            .spawn((Name::new("Lights"), SpatialBundle::default()))
            .with_children(|parent| {
                for light in rsw.objects.1.iter() {
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
                        EnvironmentalLight { range: light.range },
                    ));
                }
            })
            .id()
    }

    fn spawn_environmental_sounds(
        rsw: &rsw::RSW,
        world: &mut bevy::ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Spawning environmental sounds of {:?}", load_context.path());
        world
            .spawn((Name::new("Sounds"), SpatialBundle::default()))
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
                        EnvironmentalSound {
                            source: audio_handle,
                            volume: sound.volume,
                        },
                    ));
                }
            })
            .id()
    }

    fn spawn_environmental_effects(
        rsw: &rsw::RSW,
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
                for effect in rsw.objects.3.iter() {
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
