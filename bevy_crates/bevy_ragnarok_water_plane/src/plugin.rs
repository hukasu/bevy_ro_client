use std::borrow::Cow;

use bevy_app::{App, PreUpdate};
use bevy_asset::{AssetServer, Assets, Handle, RenderAssetUsages, uuid_handle};
use bevy_camera::primitives::Aabb;
use bevy_ecs::{
    entity::Entity,
    query::Without,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Commands, Populated, Res, ResMut},
};
use bevy_image::Image;
use bevy_math::{Vec2, Vec3, Vec3A, primitives::Plane3d};
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
                Aabb {
                    center: Vec3A::new(0., water_plane.water_level, 0.),
                    half_extents: Vec3A::new(0.5, water_plane.wave_height, 0.5),
                },
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

/// Builds the [`Mesh`] for a shape.
///
/// The shape is encoded as a [`u16`] the 4 highest bits are the
/// bottom row of a 4x4 cube, the following 4 bits are the next row, etc.
///
/// e.g., `0b1100110011000000` is
/// ```ignore
/// 0000
/// 1100
/// 1100
/// 1100
/// ```
fn build_mesh(shape: u16) -> Mesh {
    info!("Building mesh for shape {shape}");
    let (width, height, u_offset, v_offset) = match shape {
        0b1000000000000000 => (1u16, 1u16, 0., 0.),
        0b1000100000000000 => (1, 2, 0., 0.),
        0b1000100010000000 => (1, 3, 0., 0.),
        0b1000100010001000 => (1, 4, 0., 0.),
        0b1100000000000000 => (2, 1, 0., 0.),
        0b1100110000000000 => (2, 2, 0., 0.),
        0b1100110011000000 => (2, 3, 0., 0.),
        0b1100110011001100 => (2, 4, 0., 0.),
        0b1110000000000000 => (3, 1, 0., 0.),
        0b1110111000000000 => (3, 2, 0., 0.),
        0b1110111011100000 => (3, 3, 0., 0.),
        0b1110111011101110 => (3, 4, 0., 0.),
        0b1111000000000000 => (4, 1, 0., 0.),
        0b1111111100000000 => (4, 2, 0., 0.),
        0b1111111111110000 => (4, 3, 0., 0.),
        0b1111111111111111 => (4, 4, 0., 0.),
        0b0000000000001000 => (1, 1, 0., 0.75),
        0b0000000010001000 => (1, 2, 0., 0.5),
        0b0000100010001000 => (1, 3, 0., 0.25),
        0b0000000000001100 => (2, 1, 0., 0.75),
        0b0000000011001100 => (2, 2, 0., 0.5),
        0b0000110011001100 => (2, 3, 0., 0.25),
        0b0000000000001110 => (3, 1, 0., 0.75),
        0b0000000011101110 => (3, 2, 0., 0.5),
        0b0000111011101110 => (3, 3, 0., 0.25),
        0b0000000000001111 => (4, 1, 0., 0.75),
        0b0000000011111111 => (4, 2, 0., 0.5),
        0b0000111111111111 => (4, 3, 0., 0.25),
        0b0001000000000000 => (1, 1, 0.75, 0.),
        0b0001000100000000 => (1, 2, 0.75, 0.),
        0b0001000100010000 => (1, 3, 0.75, 0.),
        0b0001000100010001 => (1, 4, 0.75, 0.),
        0b0011000000000000 => (2, 1, 0.5, 0.),
        0b0011001100000000 => (2, 2, 0.5, 0.),
        0b0011001100110000 => (2, 3, 0.5, 0.),
        0b0011001100110011 => (2, 4, 0.5, 0.),
        0b0111000000000000 => (3, 1, 0.25, 0.),
        0b0111011100000000 => (3, 2, 0.25, 0.),
        0b0111011101110000 => (3, 3, 0.25, 0.),
        0b0111011101110111 => (3, 4, 0.25, 0.),
        0b0000000000000001 => (1, 1, 0.75, 0.75),
        0b0000000000010001 => (1, 2, 0.75, 0.5),
        0b0000000100010001 => (1, 3, 0.75, 0.25),
        0b0000000000000011 => (2, 1, 0.5, 0.75),
        0b0000000000110011 => (2, 2, 0.5, 0.5),
        0b0000001100110011 => (2, 3, 0.5, 0.25),
        0b0000000000000111 => (3, 1, 0.25, 0.75),
        0b0000000001110111 => (3, 2, 0.25, 0.5),
        0b0000011101110111 => (3, 3, 0.25, 0.25),
        _ => unreachable!("{shape} is a invalid shape."),
    };

    let vertices = (0..=width)
        .flat_map(|x| {
            (0..=height).map(move |z| {
                Vec3::new(
                    -(width as f32 / 2.) + (x as f32),
                    0.,
                    -(height as f32 / 2.) + (z as f32),
                )
            })
        })
        .collect::<Vec<_>>();
    let uvs = (0..=width)
        .flat_map(|x| {
            (0..=height)
                .map(move |z| Vec2::new(u_offset + 0.25 * (x as f32), v_offset + 0.25 * (z as f32)))
        })
        .collect::<Vec<_>>();
    let indices = (0..width)
        .flat_map(|x| {
            (0..height).flat_map(move |z| {
                [
                    x * (height + 1) + z,
                    (x + 1) * (height + 1) + (z + 1),
                    x * (height + 1) + (z + 1),
                    (x + 1) * (height + 1) + (z + 1),
                    x * (height + 1) + z,
                    (x + 1) * (height + 1) + z,
                ]
            })
        })
        .collect::<Vec<_>>();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::NEG_Y; vertices.len()]);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U16(indices));

    mesh
}

#[cfg(test)]
mod tests {
    use bevy_mesh::VertexAttributeValues;

    use super::*;

    #[expect(clippy::unwrap_used)]
    #[test]
    fn test_build_mesh() {
        let mesh = build_mesh(0b1000000000000000);
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            vec![
                [-0.5, 0., -0.5],
                [-0.5, 0., 0.5],
                [0.5, 0., -0.5],
                [0.5, 0., 0.5],
            ]
        );
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("Wrong values for uv");
        };
        assert_eq!(uvs, &vec![[0., 0.], [0., 0.25], [0.25, 0.], [0.25, 0.25]]);
        let Indices::U16(indices) = mesh.indices().unwrap() else {
            panic!("Wrong type of indices");
        };
        assert_eq!(indices, &vec![0, 3, 1, 3, 0, 2]);

        let mesh = build_mesh(0b0001000000000000);
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            vec![
                [-0.5, 0., -0.5],
                [-0.5, 0., 0.5],
                [0.5, 0., -0.5],
                [0.5, 0., 0.5],
            ]
        );
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("Wrong values for uv");
        };
        assert_eq!(uvs, &vec![[0.75, 0.], [0.75, 0.25], [1., 0.], [1., 0.25]]);

        let mesh = build_mesh(0b0000000000001000);
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            vec![
                [-0.5, 0., -0.5],
                [-0.5, 0., 0.5],
                [0.5, 0., -0.5],
                [0.5, 0., 0.5],
            ]
        );
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("Wrong values for uv");
        };
        assert_eq!(uvs, &vec![[0., 0.75], [0., 1.], [0.25, 0.75], [0.25, 1.]]);

        let mesh = build_mesh(0b0000000000000001);
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            vec![
                [-0.5, 0., -0.5],
                [-0.5, 0., 0.5],
                [0.5, 0., -0.5],
                [0.5, 0., 0.5],
            ]
        );
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("Wrong values for uv");
        };
        assert_eq!(uvs, &vec![[0.75, 0.75], [0.75, 1.], [1., 0.75], [1., 1.]]);

        let mesh = build_mesh(0b0000000000110011);
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            vec![
                [-1., 0., -1.],
                [-1., 0., 0.],
                [-1., 0., 1.],
                [0., 0., -1.],
                [0., 0., 0.],
                [0., 0., 1.],
                [1., 0., -1.],
                [1., 0., 0.],
                [1., 0., 1.],
            ]
        );
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("Wrong values for uv");
        };
        assert_eq!(
            uvs,
            &vec![
                [0.5, 0.5],
                [0.5, 0.75],
                [0.5, 1.],
                [0.75, 0.5],
                [0.75, 0.75],
                [0.75, 1.],
                [1., 0.5],
                [1., 0.75],
                [1., 1.],
            ]
        );

        let mesh = build_mesh(0b1111111111111111);
        assert_eq!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap(),
            vec![
                [-2., 0., -2.],
                [-2., 0., -1.],
                [-2., 0., 0.],
                [-2., 0., 1.],
                [-2., 0., 2.],
                [-1., 0., -2.],
                [-1., 0., -1.],
                [-1., 0., 0.],
                [-1., 0., 1.],
                [-1., 0., 2.],
                [0., 0., -2.],
                [0., 0., -1.],
                [0., 0., 0.],
                [0., 0., 1.],
                [0., 0., 2.],
                [1., 0., -2.],
                [1., 0., -1.],
                [1., 0., 0.],
                [1., 0., 1.],
                [1., 0., 2.],
                [2., 0., -2.],
                [2., 0., -1.],
                [2., 0., 0.],
                [2., 0., 1.],
                [2., 0., 2.],
            ]
        );
        let VertexAttributeValues::Float32x2(uvs) = mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap()
        else {
            panic!("Wrong values for uv");
        };
        assert_eq!(
            uvs,
            &vec![
                [0., 0.],
                [0., 0.25],
                [0., 0.5],
                [0., 0.75],
                [0., 1.],
                [0.25, 0.],
                [0.25, 0.25],
                [0.25, 0.5],
                [0.25, 0.75],
                [0.25, 1.],
                [0.5, 0.],
                [0.5, 0.25],
                [0.5, 0.5],
                [0.5, 0.75],
                [0.5, 1.],
                [0.75, 0.],
                [0.75, 0.25],
                [0.75, 0.5],
                [0.75, 0.75],
                [0.75, 1.],
                [1., 0.],
                [1., 0.25],
                [1., 0.5],
                [1., 0.75],
                [1., 1.],
            ]
        );
    }
}
