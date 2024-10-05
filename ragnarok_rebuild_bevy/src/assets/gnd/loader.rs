use std::collections::BTreeMap;

use bevy::{
    asset::{io::Reader, AsyncReadExt, Handle, LoadContext},
    core::Name,
    ecs::world::World,
    hierarchy::BuildWorldChildren,
    math::{Vec2, Vec3},
    pbr::{MaterialMeshBundle, NotShadowCaster, NotShadowReceiver},
    prelude::{Entity, SpatialBundle},
    render::{
        mesh::{Indices, Mesh, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        texture::{
            Image, ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
            ImageSamplerDescriptor,
        },
    },
    scene::Scene,
};

use ragnarok_rebuild_assets::{common, gnd};
use serde::{Deserialize, Serialize};

use crate::{
    assets::{paths, water_plane},
    materials::{GndMaterial, WaterPlaneMaterial},
};

use super::{components::Ground, GroundScale};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetLoaderSettings {
    pub water_plane: Option<common::WaterPlane>,
}

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = AssetLoaderSettings;
    type Error = gnd::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            bevy::log::trace!("Loading Gnd {:?}.", load_context.path());

            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let gnd = gnd::GND::from_reader(&mut data.as_slice())?;

            Ok(Self::generate_ground(&gnd, settings, load_context))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gnd"]
    }
}

impl AssetLoader {
    fn generate_ground(
        gnd: &gnd::GND,
        settings: &AssetLoaderSettings,
        load_context: &mut LoadContext,
    ) -> Scene {
        let mut world = World::new();

        // 2x2 tiles per gnd cube
        world.insert_resource(GroundScale(2. / gnd.scale));
        Self::generate_ground_mesh(gnd, &mut world, load_context);
        Self::generate_water_planes(gnd, settings, &mut world, load_context);

        Scene::new(world)
    }

    fn generate_ground_mesh(gnd: &gnd::GND, world: &mut World, load_context: &mut LoadContext) {
        let textures: Vec<Handle<Image>> = gnd
            .textures
            .iter()
            .map(|path| load_context.load(format!("{}{}", paths::TEXTURE_FILES_FOLDER, path)))
            .collect();
        let materials: Vec<Handle<GndMaterial>> = textures
            .iter()
            .cloned()
            .map(Self::mesh_material)
            .enumerate()
            .map(|(i, material)| load_context.add_labeled_asset(format!("Material{i}"), material))
            .collect();
        Self::build_cubes(gnd, &materials, world, load_context);
    }

    fn generate_water_planes(
        gnd: &gnd::GND,
        settings: &AssetLoaderSettings,
        world: &mut World,
        load_context: &mut LoadContext,
    ) {
        let water_planes = world
            .spawn((Name::new("WaterPlanes"), SpatialBundle::default()))
            .id();

        let rsw_water_plane = [&settings.water_plane].into_iter().filter_map(|plane| {
            plane
                .as_ref()
                .map(|plane| ("RswWaterPlane".to_owned(), plane))
        });
        let gnd_water_planes = gnd
            .water_planes
            .iter()
            .enumerate()
            .map(|(i, plane)| (format!("WaterPlane{}", i), plane));
        for (name, water_plane) in rsw_water_plane.chain(gnd_water_planes) {
            let id = Self::build_water_plane(gnd, water_plane, &name, world, load_context);
            world.entity_mut(water_planes).add_child(id);
        }
    }

    fn build_cubes(
        gnd: &gnd::GND,
        materials: &[Handle<GndMaterial>],
        world: &mut World,
        load_context: &mut LoadContext,
    ) {
        let mut groups = vec![(vec![], vec![]); materials.len()];
        for (i, cube) in gnd.ground_mesh_cubes.iter().enumerate() {
            if cube.upwards_facing_surface >= 0 {
                Self::build_up_face(gnd, cube, &mut groups, i as i32);
            };
            if cube.east_facing_surface >= 0 {
                Self::build_east_face(
                    gnd,
                    cube,
                    &gnd.ground_mesh_cubes[i + 1],
                    &mut groups,
                    i as i32,
                );
            };
            if cube.north_facing_surface >= 0 {
                Self::build_north_face(
                    gnd,
                    cube,
                    &gnd.ground_mesh_cubes[i + gnd.width as usize],
                    &mut groups,
                    i as i32,
                );
            };
        }

        let asset_usage = if cfg!(feature = "debug") {
            RenderAssetUsages::all()
        } else {
            RenderAssetUsages::RENDER_WORLD
        };

        let meshs = groups
            .into_iter()
            .enumerate()
            .map(|(i, (vertex, uvs))| {
                load_context.add_labeled_asset(
                    format!("Primitive{i}"),
                    Mesh::new(PrimitiveTopology::TriangleList, asset_usage)
                        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex)
                        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                        .with_computed_flat_normals(),
                )
            })
            .collect::<Vec<_>>();

        world
            .spawn((Name::new("Primitives"), SpatialBundle::default(), Ground))
            .with_children(|parent| {
                for (i, mesh) in meshs.iter().enumerate() {
                    parent.spawn((
                        Name::new(format!("Primitive{i}")),
                        MaterialMeshBundle {
                            mesh: mesh.clone(),
                            material: materials[i].clone(),
                            ..Default::default()
                        },
                    ));
                }
            });
    }

    fn build_up_face(
        gnd: &gnd::GND,
        cube: &gnd::GroundMeshCube,
        groups: &mut [(Vec<Vec3>, Vec<Vec2>)],
        i: i32,
    ) {
        let surface = &gnd.surfaces[cube.upwards_facing_surface as usize];
        let (vertex, uvs) = &mut groups[surface.texture_id as usize];

        // Referent to the current cube
        let bottom_left_x = ((i % gnd.width as i32) - (gnd.width as i32 / 2)) as f32 * gnd.scale;
        let bottom_left_z = ((i / gnd.width as i32) - (gnd.height as i32 / 2)) as f32 * gnd.scale;

        vertex.push(Vec3::new(
            bottom_left_x,
            cube.bottom_left_height,
            bottom_left_z,
        ));
        vertex.push(Vec3::new(
            bottom_left_x + gnd.scale,
            cube.bottom_right_height,
            bottom_left_z,
        ));
        vertex.push(Vec3::new(
            bottom_left_x,
            cube.top_left_height,
            bottom_left_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_left));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_left));

        vertex.push(Vec3::new(
            bottom_left_x + gnd.scale,
            cube.bottom_right_height,
            bottom_left_z,
        ));
        vertex.push(Vec3::new(
            bottom_left_x + gnd.scale,
            cube.top_right_height,
            bottom_left_z + gnd.scale,
        ));
        vertex.push(Vec3::new(
            bottom_left_x,
            cube.top_left_height,
            bottom_left_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_right));
        uvs.push(Vec2::from_array(surface.top_left));
    }

    fn build_east_face(
        gnd: &gnd::GND,
        cube: &gnd::GroundMeshCube,
        east_cube: &gnd::GroundMeshCube,
        groups: &mut [(Vec<Vec3>, Vec<Vec2>)],
        i: i32,
    ) {
        let surface = &gnd.surfaces[cube.east_facing_surface as usize];
        let (vertex, uvs) = &mut groups[surface.texture_id as usize];

        // Referent to the current cube
        let bottom_right_x =
            (((i + 1) % gnd.width as i32) - (gnd.width as i32 / 2)) as f32 * gnd.scale;
        let bottom_right_z = ((i / gnd.width as i32) - (gnd.height as i32 / 2)) as f32 * gnd.scale;

        vertex.push(Vec3::new(
            bottom_right_x,
            cube.bottom_right_height,
            bottom_right_z,
        ));
        vertex.push(Vec3::new(
            bottom_right_x,
            east_cube.bottom_left_height,
            bottom_right_z,
        ));
        vertex.push(Vec3::new(
            bottom_right_x,
            cube.top_right_height,
            bottom_right_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_right));
        uvs.push(Vec2::from_array(surface.bottom_left));

        vertex.push(Vec3::new(
            bottom_right_x,
            cube.top_right_height,
            bottom_right_z + gnd.scale,
        ));
        vertex.push(Vec3::new(
            bottom_right_x,
            east_cube.bottom_left_height,
            bottom_right_z,
        ));
        vertex.push(Vec3::new(
            bottom_right_x,
            east_cube.top_left_height,
            bottom_right_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_left));
        uvs.push(Vec2::from_array(surface.top_right));
        uvs.push(Vec2::from_array(surface.top_left));
    }

    fn build_north_face(
        gnd: &gnd::GND,
        cube: &gnd::GroundMeshCube,
        north_cube: &gnd::GroundMeshCube,
        groups: &mut [(Vec<Vec3>, Vec<Vec2>)],
        i: i32,
    ) {
        let surface = &gnd.surfaces[cube.north_facing_surface as usize];
        let (vertex, uvs) = &mut groups[surface.texture_id as usize];

        // Referent to the current cube
        let top_left_x = ((i % gnd.width as i32) - (gnd.width as i32 / 2)) as f32 * gnd.scale;
        let top_left_z = (((i + gnd.width as i32) / gnd.width as i32) - (gnd.height as i32 / 2))
            as f32
            * gnd.scale;

        vertex.push(Vec3::new(top_left_x, cube.top_left_height, top_left_z));
        vertex.push(Vec3::new(
            top_left_x + gnd.scale,
            cube.top_right_height,
            top_left_z,
        ));
        vertex.push(Vec3::new(
            top_left_x,
            north_cube.bottom_left_height,
            top_left_z,
        ));
        uvs.push(Vec2::from_array(surface.bottom_left));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_left));

        vertex.push(Vec3::new(
            top_left_x,
            north_cube.bottom_left_height,
            top_left_z,
        ));
        vertex.push(Vec3::new(
            top_left_x + gnd.scale,
            cube.top_right_height,
            top_left_z,
        ));
        vertex.push(Vec3::new(
            top_left_x + gnd.scale,
            north_cube.bottom_right_height,
            top_left_z,
        ));
        uvs.push(Vec2::from_array(surface.top_left));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_right));
    }

    #[must_use]
    fn build_water_plane(
        gnd: &gnd::GND,
        water_plane: &common::WaterPlane,
        name: &str,
        world: &mut World,
        load_context: &mut LoadContext,
    ) -> Entity {
        bevy::log::trace!("Generating {} for {:?}", name, load_context.path());
        let mesh: Handle<Mesh> = load_context.add_labeled_asset(
            format!("{}Mesh", name),
            Self::water_plane_mesh(gnd, water_plane),
        );
        let material: [Handle<WaterPlaneMaterial>; 32] = std::array::from_fn(|i| {
            Self::water_plane_material(load_context, name, i, water_plane.water_type)
        });

        world
            .spawn((
                Name::new(name.to_owned()),
                SpatialBundle::default(),
                mesh,
                water_plane::WaterPlane::new(material, water_plane.texture_cyclical_interval),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id()
    }

    #[must_use]
    fn mesh_material(texture: Handle<Image>) -> GndMaterial {
        GndMaterial {
            color_texture: texture,
        }
    }

    #[must_use]
    fn water_plane_mesh(gnd: &gnd::GND, water_plane: &common::WaterPlane) -> Mesh {
        let mut vertices: Vec<Vec3> = vec![];
        let mut normals: Vec<Vec3> = vec![];
        let mut uvs: Vec<Vec2> = vec![];
        let mut indices: Vec<u16> = vec![];

        let mut insert_vertex = |x: u32, z: u32| {
            let vertex_x = (x as i32 - ((gnd.width / 2) as i32)) as f32 * gnd.scale;
            let vertex_z = (z as i32 - ((gnd.height / 2) as i32)) as f32 * gnd.scale;
            vertices.push(Vec3::new(vertex_x, water_plane.water_level, vertex_z));
            normals.push(Vec3::NEG_Y);
            uvs.push(Vec2::new(x as f32, (gnd.height - z) as f32));
            vertices.len() as u16 - 1
        };

        let mut tile_cache: BTreeMap<(u32, u32), (u16, u16, u16, u16)> = BTreeMap::new();
        for (i, _) in gnd
            .ground_mesh_cubes
            .iter()
            .enumerate()
            .filter(|(_, cube)| {
                cube.upwards_facing_surface != -1
                    && [
                        cube.top_left_height,
                        cube.top_right_height,
                        cube.bottom_left_height,
                        cube.bottom_right_height,
                    ]
                    .iter()
                    .any(|height| height >= &water_plane.water_level)
            })
        {
            let x = i as u32 % gnd.width;
            let z = i as u32 / gnd.width;

            let bottom_left = if z > 0 && tile_cache.contains_key(&(x, z - 1)) {
                // If tile to the south is already cached, use its index
                let Some(cache) = tile_cache.get(&(x, z - 1)) else {
                    unreachable!("Check if contains key returned true but could not get value.");
                };
                cache.2
            } else if x > 0 && tile_cache.contains_key(&(x - 1, z)) {
                // If tile to the west is already cached, use its index
                let Some(cache) = tile_cache.get(&(x - 1, z)) else {
                    unreachable!("Check if contains key returned true but could not get value.");
                };
                cache.1
            } else {
                insert_vertex(x, z)
            };
            let bottom_right = if z > 0 && tile_cache.contains_key(&(x, z - 1)) {
                // If tile to the south is already cached, use its index
                let Some(cache) = tile_cache.get(&(x, z - 1)) else {
                    unreachable!("Check if contains key returned true but could not get value.");
                };
                cache.3
            } else {
                insert_vertex(x + 1, z)
            };
            let top_left = if x > 0 && tile_cache.contains_key(&(x - 1, z)) {
                // If tile to the west is already cached, use its index
                let Some(cache) = tile_cache.get(&(x - 1, z)) else {
                    unreachable!("Check if contains key returned true but could not get value.");
                };
                cache.3
            } else {
                insert_vertex(x, z + 1)
            };
            let top_right = insert_vertex(x + 1, z + 1);

            tile_cache.insert((x, z), (bottom_left, bottom_right, top_left, top_right));
            indices.extend(&[
                bottom_left,
                bottom_right,
                top_left,
                bottom_right,
                top_right,
                top_left,
            ]);
        }
        // dbg!(tile_cache);
        let asset_usage = if cfg!(feature = "debug") {
            RenderAssetUsages::all()
        } else {
            RenderAssetUsages::RENDER_WORLD
        };
        Mesh::new(PrimitiveTopology::TriangleList, asset_usage)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
            .with_inserted_indices(Indices::U16(indices))
    }

    #[must_use]
    fn water_plane_material(
        load_context: &mut LoadContext,
        name: &str,
        frame: usize,
        water_type: i32,
    ) -> Handle<WaterPlaneMaterial> {
        load_context.labeled_asset_scope(
            format!("{}Material/Frame{}", name, frame),
            |load_context| {
                let image: Handle<Image> = load_context
                    .loader()
                    .with_settings(|m: &mut ImageLoaderSettings| {
                        m.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            label: Some("WaterSample".to_owned()),
                            address_mode_u: ImageAddressMode::Repeat,
                            address_mode_v: ImageAddressMode::Repeat,
                            address_mode_w: ImageAddressMode::Repeat,
                            mag_filter: ImageFilterMode::Linear,
                            min_filter: ImageFilterMode::Linear,
                            ..Default::default()
                        })
                    })
                    .load(format!(
                        "{}water{}{:02}.jpg",
                        paths::WATER_TEXTURE_FILES_FOLDER,
                        water_type,
                        frame
                    ));
                WaterPlaneMaterial { texture: image }
            },
        )
    }
}
