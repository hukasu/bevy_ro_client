use std::borrow::Cow;

use bevy_app::{App, PreUpdate};
use bevy_asset::{AssetServer, Assets, Handle, RenderAssetUsages, uuid_handle};
use bevy_ecs::{
    entity::Entity,
    query::Without,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Commands, Populated, Res, ResMut},
};
use bevy_image::Image;
use bevy_math::{Vec2, Vec3, primitives::Plane3d};
use bevy_mesh::{Mesh, Mesh3d, MeshBuilder, Meshable};
use bevy_pbr::MeshMaterial3d;
use bevy_platform::collections::HashMap;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::{
    WaterPlane,
    material::{self, WaterPlaneMaterial, Wave},
};

const WATER_PLANE_MESH: Handle<Mesh> = uuid_handle!("7a77a34b-40ea-42ec-b935-1b57b38b17d7");

pub struct Plugin {
    pub texture_prefix: Cow<'static, str>,
}

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaterPlaneTypeImages {
            prefix: self.texture_prefix.clone(),
            materials: Default::default(),
            images: Default::default(),
            loading: Default::default(),
        });

        // Systems
        app.add_systems(
            PreUpdate,
            (build_texture_array, prepare_water_plane).chain(),
        );

        // Material
        app.add_plugins(material::plugin::Plugin);

        // Register Types
        app.register_type::<WaterPlane>();
    }

    fn finish(&self, app: &mut App) {
        if let Err(err) = app.world_mut().resource_mut::<Assets<Mesh>>().insert(
            &WATER_PLANE_MESH,
            Plane3d::new(Vec3::NEG_Y, Vec2::splat(0.5))
                .mesh()
                .subdivisions(4)
                .build(),
        ) {
            unreachable!("Should never error for Uuid handles. `{err}`");
        };
    }
}

#[derive(Resource)]
struct WaterPlaneTypeImages {
    prefix: Cow<'static, str>,
    materials: HashMap<i32, Handle<WaterPlaneMaterial>>,
    images: HashMap<i32, Handle<Image>>,
    loading: HashMap<i32, [Handle<Image>; 32]>,
}

fn prepare_water_plane(
    mut commands: Commands,
    water_planes: Populated<(Entity, &WaterPlane), Without<MeshMaterial3d<WaterPlaneMaterial>>>,
    asset_server: Res<AssetServer>,
    mut water_plane_materials: ResMut<Assets<WaterPlaneMaterial>>,
    mut water_plane_type_images: ResMut<WaterPlaneTypeImages>,
) {
    for (entity, water_plane) in water_planes.into_inner() {
        if let Some(water_plane_material) = water_plane_type_images
            .materials
            .get(&water_plane.water_type)
        {
            commands.entity(entity).insert((
                Mesh3d(WATER_PLANE_MESH.clone()),
                MeshMaterial3d(water_plane_material.clone()),
            ));
        } else if let Some(texture_array) =
            water_plane_type_images.images.get(&water_plane.water_type)
        {
            let material = WaterPlaneMaterial {
                texture: texture_array.clone(),
                wave: Wave {
                    water_level: water_plane.water_level,
                    wave_height: water_plane.wave_height,
                    wave_speed: water_plane.wave_speed,
                    wave_pitch: water_plane.wave_pitch,
                    frames_per_second: 60. / water_plane.texture_cyclical_interval as f32,
                },
                opaque: water_plane.water_type == 4 || water_plane.water_type == 6,
            };
            let material_handle = water_plane_materials.add(material);
            water_plane_type_images
                .images
                .remove(&water_plane.water_type);
            water_plane_type_images
                .materials
                .insert(water_plane.water_type, material_handle);
        } else if !water_plane_type_images
            .loading
            .contains_key(&water_plane.water_type)
        {
            let WaterPlaneTypeImages {
                prefix,
                materials: _,
                images: _,
                loading,
            } = water_plane_type_images.as_mut();
            loading.insert(
                water_plane.water_type,
                std::array::from_fn(|i| {
                    asset_server.load(format!(
                        "{}water{}{:02}.jpg",
                        prefix, water_plane.water_type, i
                    ))
                }),
            );
        }
    }
}

fn build_texture_array(
    mut water_plane_type_images: ResMut<WaterPlaneTypeImages>,
    mut images: ResMut<Assets<Image>>,
) {
    let WaterPlaneTypeImages {
        prefix: _,
        materials: _,
        images: water_plane_texture_array,
        loading,
    } = water_plane_type_images.as_mut();

    let mut completed: Vec<i32> = vec![];
    for (water_type, loading_images) in loading.iter() {
        let loading_images = loading_images.clone().map(|image| images.get(image.id()));
        if loading_images.iter().all(Option::is_some) {
            let loading_images = loading_images.map(Option::unwrap);

            let images_size = loading_images[0].size();
            debug_assert!(
                loading_images
                    .iter()
                    .all(|image| image.size() == images_size)
            );

            let magenta = (0..(images_size.x * images_size.y * 4))
                .map(|i| if i % 4 == 1 { 0u8 } else { 255u8 })
                .collect::<Vec<_>>();

            let image_array = Image::new(
                Extent3d {
                    width: images_size.x,
                    height: images_size.y,
                    depth_or_array_layers: 32,
                },
                TextureDimension::D2,
                loading_images
                    .iter()
                    .flat_map(|image| image.data.as_deref().unwrap_or(&magenta))
                    .copied()
                    .collect(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );

            water_plane_texture_array.insert(*water_type, images.add(image_array));
            completed.push(*water_type);
        }
    }

    for water_type in completed {
        loading.remove(&water_type);
    }
}
