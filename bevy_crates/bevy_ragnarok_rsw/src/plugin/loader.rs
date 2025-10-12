use std::{path::PathBuf, time::Duration};

use bevy_asset::{Handle, LoadContext, io::Reader};
use bevy_audio::AudioSource;
use bevy_camera::{primitives::Aabb, visibility::Visibility};
use bevy_color::{Color, Luminance};
use bevy_ecs::{
    entity::Entity,
    hierarchy::{ChildOf, Children},
    name::Name,
    relationship::Relationship,
    spawn::{SpawnRelated, SpawnableList},
};
use bevy_light::{AmbientLight, DirectionalLight, PointLight};
use bevy_math::{EulerRot, Quat, Vec3, Vec3A};
use bevy_ragnarok_quad_tree::QuadTreeNode;
use bevy_scene::{Scene, SceneRoot};
use bevy_time::Timer;
use bevy_transform::components::Transform;
use ragnarok_rsw::{Model, Rsw, quad_tree::Crawler};

use crate::{
    Altitude, AnimatedProp, DiffuseLight, EnvironmentalEffect, EnvironmentalLight,
    EnvironmentalSound, World, WorldQuadTree, assets::RswWorld,
};

pub struct AssetLoader {
    /// Prefix for .rsm files
    pub model_path_prefix: PathBuf,
    /// Prefix for .gnd files
    pub ground_path_prefix: PathBuf,
    /// Prefix for .gat files
    pub altitude_path_prefix: PathBuf,
    /// Prefix for .wav files
    pub sound_path_prefix: PathBuf,
}

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = RswWorld;
    type Settings = ();
    type Error = ragnarok_rsw::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let rsw = Rsw::from_reader(&mut data.as_slice())?;

        Ok(self.generate_world_scene(&rsw, load_context))
    }

    fn extensions(&self) -> &[&str] {
        &["rsw"]
    }
}

impl AssetLoader {
    const UNNAMED_RSW: &str = "Unnamed Rsw";
    const LIGHT_LUX: f32 = 2000.;
    const POINT_LIGHT_LUMEN: f32 = 5_000_000.;

    fn generate_world_scene(&self, rsw: &Rsw, load_context: &mut LoadContext) -> RswWorld {
        log::trace!("Generating {:?} world.", load_context.path());

        let mut world = bevy_ecs::world::World::new();

        Self::set_ambient_light(rsw, &mut world, load_context);
        let directional_light = Self::spawn_directional_light(rsw, &mut world, load_context);
        let ground = self.spawn_ground(rsw, &mut world, load_context);
        let tiles = self.spawn_tiles(rsw, &mut world, load_context);
        let animated_props = self.spawn_animated_props(rsw, &mut world, load_context);
        let environmental_lights = Self::spawn_environmental_lights(rsw, &mut world, load_context);
        let environmental_sounds = self.spawn_environmental_sounds(rsw, &mut world, load_context);
        let environmental_effects =
            Self::spawn_environmental_effects(rsw, &mut world, load_context);

        let filename = match load_context.path().file_name() {
            Some(filename) => filename.to_str().unwrap_or(Self::UNNAMED_RSW),
            None => Self::UNNAMED_RSW,
        };

        let rsw_world = world
            .spawn((
                Name::new(filename.to_string()),
                Transform::default(),
                Visibility::default(),
                World,
            ))
            .add_children(&[
                directional_light,
                ground,
                tiles,
                animated_props,
                environmental_lights,
                environmental_sounds,
                environmental_effects,
            ])
            .id();

        Self::spawn_quad_tree(rsw, rsw_world, &mut world, load_context);

        RswWorld {
            scene: load_context.add_labeled_asset("Scene".into(), Scene::new(world)),
        }
    }

    fn set_ambient_light(
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) {
        log::trace!("Setting ambient light of {:?}.", load_context.path());
        let color = Color::srgb(
            rsw.lighting_parameters.ambient_color[0],
            rsw.lighting_parameters.ambient_color[1],
            rsw.lighting_parameters.ambient_color[2],
        );
        world.insert_resource(AmbientLight {
            brightness: Self::LIGHT_LUX
                * (color.luminance() + (1. - rsw.lighting_parameters.shadow_map_alpha)),
            color,
            affects_lightmapped_meshes: true,
        });
    }

    fn spawn_directional_light(
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        log::trace!("Spawning directional light of {:?}", load_context.path());
        let base_distance = -500.;
        let latitude_radians = (rsw.lighting_parameters.latitude as f32).to_radians();
        let longitude_radians = (rsw.lighting_parameters.longitude as f32).to_radians();

        let mut light_transform = Transform::from_xyz(0., base_distance, 0.);
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_x(longitude_radians));
        light_transform.rotate_around(Vec3::ZERO, Quat::from_rotation_y(latitude_radians));

        let directional_light_color = Color::srgb(
            rsw.lighting_parameters.diffuse_color[0],
            rsw.lighting_parameters.diffuse_color[1],
            rsw.lighting_parameters.diffuse_color[2],
        );
        world
            .spawn((
                Name::new("DirectionalLight"),
                DiffuseLight,
                DirectionalLight {
                    color: directional_light_color,
                    illuminance: Self::LIGHT_LUX
                        * (directional_light_color.luminance()
                            + rsw.lighting_parameters.shadow_map_alpha),
                    shadows_enabled: true,
                    ..Default::default()
                },
                light_transform.looking_at(Vec3::ZERO, Vec3::Y),
            ))
            .id()
    }

    fn spawn_ground(
        &self,
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        log::trace!("Spawning ground of {:?}", load_context.path());

        let world_ground = load_context.loader().load(format!(
            "{}{}",
            self.ground_path_prefix.display(),
            rsw.gnd_file
        ));

        world
            .spawn((Name::new(rsw.gnd_file.to_string()), SceneRoot(world_ground)))
            .id()
    }

    fn spawn_tiles(
        &self,
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        log::trace!("Spawning tiles of {:?}", load_context.path());

        let world_tiles = load_context
            .loader()
            .with_settings(|settings: &mut f32| {
                *settings = 5.;
            })
            .load(format!(
                "{}{}#Scene",
                self.altitude_path_prefix.display(),
                rsw.gat_file
            ));

        world
            .spawn((
                Name::new(rsw.gat_file.to_string()),
                Altitude,
                SceneRoot(world_tiles),
            ))
            .id()
    }

    fn spawn_animated_props(
        &self,
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        log::trace!("Spawning animated props of {:?}", load_context.path());
        world
            .spawn((
                Name::new("Models"),
                Transform::default(),
                Visibility::default(),
                Children::spawn(ModelSpawner::new(&rsw.models, self, load_context)),
            ))
            .id()
    }

    fn spawn_environmental_lights(
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        log::trace!("Spawning environmental lights of {:?}", load_context.path());
        world
            .spawn((
                Name::new("Lights"),
                Transform::default(),
                Visibility::default(),
            ))
            .with_children(|parent| {
                for light in rsw.lights.iter() {
                    let color = Color::srgb(light.color[0], light.color[1], light.color[2]);
                    parent.spawn((
                        Name::new(light.name.to_string()),
                        EnvironmentalLight { range: light.range },
                        PointLight {
                            color,
                            intensity: Self::POINT_LIGHT_LUMEN
                                * (color.luminance()
                                    + (1. - rsw.lighting_parameters.shadow_map_alpha)),
                            range: light.range / 5.,
                            shadows_enabled: true,
                            ..Default::default()
                        },
                        Aabb::from_min_max(-Vec3::splat(light.range), Vec3::splat(light.range)),
                        Transform::from_translation(Vec3::from_array(light.position)),
                    ));
                }
            })
            .id()
    }

    fn spawn_environmental_sounds(
        &self,
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        log::trace!("Spawning environmental sounds of {:?}", load_context.path());
        world
            .spawn((Name::new("Sounds"), Transform::default(), Visibility::default()))
            .with_children(|parent| {
                for sound in rsw.sounds.iter() {
                    let audio_handle: Handle<AudioSource> =
                        load_context.load(self.sound_path_prefix.join( sound.filename.as_ref()));

                    parent.spawn((
                        Name::new(sound.name.to_string()),
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
                                    log::warn!("{} had cycle set to 0. seconds. Changing to default 4. seconds.", sound.name);
                                    4.
                                } else {
                                    sound.cycle
                                }),
                                bevy_time::TimerMode::Repeating,
                            ),
                        },
                        Transform::from_translation(Vec3::from_array(sound.position)),
                    ));
                }
            })
            .id()
    }

    fn spawn_environmental_effects(
        rsw: &Rsw,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) -> Entity {
        log::trace!(
            "Spawning environmental effects of {:?}",
            load_context.path()
        );
        world
            .spawn((
                Name::new("Effects"),
                Transform::default(),
                Visibility::default(),
            ))
            .with_children(|parent| {
                for effect in rsw.effects.iter() {
                    parent.spawn((
                        Name::new(effect.name.to_string()),
                        EnvironmentalEffect,
                        Transform::from_translation(Vec3::from_array(effect.position)),
                    ));
                }
            })
            .id()
    }

    fn spawn_quad_tree(
        rsw: &Rsw,
        rsw_world: Entity,
        world: &mut bevy_ecs::world::World,
        load_context: &mut LoadContext,
    ) {
        log::trace!("Spawning quad tree of {:?}", load_context.path());

        fn recursive(
            world: &mut bevy_ecs::world::World,
            crawler: Crawler<'_>,
            parent: Entity,
        ) -> Entity {
            let node = world
                .spawn((
                    Aabb {
                        center: Vec3A::from_array(crawler.center),
                        half_extents: Vec3A::from_array(crawler.radius),
                    },
                    Transform::default(),
                    Visibility::default(),
                    ChildOf(parent),
                    <QuadTreeNode as Relationship>::from(parent),
                ))
                .id();

            if let Some([bl, br, tl, tr]) = crawler.children() {
                recursive(world, bl, node);
                recursive(world, br, node);
                recursive(world, tl, node);
                recursive(world, tr, node);
            }

            node
        }

        let crawler = rsw.quad_tree.crawl();

        let root = recursive(world, crawler, rsw_world);
        world
            .entity_mut(root)
            .insert((Name::new("QuadTree"), WorldQuadTree))
            .remove::<QuadTreeNode>();
    }
}

struct ModelSpawner {
    models: Vec<SpawningModel>,
}

struct SpawningModel {
    name: Name,
    animated_prop: AnimatedProp,
    scene: Handle<Scene>,
    transform: Transform,
}

impl ModelSpawner {
    pub fn new(models: &[Model], loader: &AssetLoader, load_context: &mut LoadContext<'_>) -> Self {
        let mut res = Vec::with_capacity(models.len());

        for model in models {
            let fix_up = if model.filename.ends_with("rsm2") {
                Vec3::new(1., -1., 1.)
            } else {
                Vec3::ONE
            };

            let model_path = loader
                .model_path_prefix
                .join(format!("{}#Scene", model.filename));
            let scene = load_context.load(model_path.to_string_lossy().to_string());

            res.push(SpawningModel {
                name: Name::new(model.name.to_string()),
                animated_prop: AnimatedProp {
                    animation_type: model.animation_type,
                    animation_speed: model.animation_speed,
                },
                scene,
                transform: Transform {
                    translation: Vec3::from_array(model.position),
                    rotation: Quat::from_euler(
                        EulerRot::ZXY,
                        model.rotation[2].to_radians(),
                        model.rotation[0].to_radians(),
                        model.rotation[1].to_radians(),
                    ),
                    scale: Vec3::from_array(model.scale) * fix_up,
                },
            });
        }

        Self { models: res }
    }
}

impl SpawnableList<ChildOf> for ModelSpawner {
    fn spawn(
        this: bevy_ecs::ptr::MovingPtr<'_, Self>,
        world: &mut bevy_ecs::world::World,
        entity: Entity,
    ) {
        for item in &this.models {
            world.spawn((
                ChildOf(entity),
                item.animated_prop,
                item.name.clone(),
                item.transform,
                SceneRoot(item.scene.clone()),
            ));
        }
    }

    fn size_hint(&self) -> usize {
        self.models.len()
    }
}
