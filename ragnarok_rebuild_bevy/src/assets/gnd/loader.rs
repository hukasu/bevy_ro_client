use std::collections::BTreeMap;

use bevy::{
    asset::{io::Reader, Handle, LoadContext},
    core::Name,
    ecs::world::World,
    image::{
        Image, ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    math::{Vec2, Vec3},
    pbr::{MeshMaterial3d, NotShadowCaster, NotShadowReceiver},
    prelude::{BuildChildren, Entity, Mesh3d, Transform, Visibility},
    render::{
        mesh::{Indices, Mesh, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        storage::ShaderStorageBuffer,
    },
    scene::Scene,
};

use ragnarok_rebuild_assets::{common, gnd};
use serde::{Deserialize, Serialize};

use crate::{
    assets::{paths, water_plane},
    helper,
};

use super::{components::Ground, material::GndMaterial, GroundScale};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetLoaderSettings {
    pub water_plane: Option<common::WaterPlane>,
}

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = AssetLoaderSettings;
    type Error = gnd::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy::log::trace!("Loading Gnd {:?}.", load_context.path());

        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let gnd = gnd::Gnd::from_reader(&mut data.as_slice())?;

        Ok(Self::generate_ground(&gnd, settings, load_context).await)
    }

    fn extensions(&self) -> &[&str] {
        &["gnd"]
    }
}

impl AssetLoader {
    async fn generate_ground(
        gnd: &gnd::Gnd,
        settings: &AssetLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Scene {
        let mut world = World::new();

        // 2x2 tiles per gnd cube
        world.insert_resource(GroundScale(2. / gnd.scale));
        Self::generate_ground_mesh(gnd, &mut world, load_context).await;
        Self::generate_water_planes(gnd, settings, &mut world, load_context);

        Scene::new(world)
    }

    async fn generate_ground_mesh(
        gnd: &gnd::Gnd,
        world: &mut World,
        load_context: &mut LoadContext<'_>,
    ) {
        let texture_atlas = Self::build_ground_texture_atlas(load_context, &gnd.textures).await;
        Self::build_cubes(gnd, texture_atlas, world, load_context);
    }

    fn generate_water_planes(
        gnd: &gnd::Gnd,
        settings: &AssetLoaderSettings,
        world: &mut World,
        load_context: &mut LoadContext,
    ) {
        let water_planes = world
            .spawn((
                Name::new("WaterPlanes"),
                Transform::default(),
                Visibility::default(),
            ))
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
        gnd: &gnd::Gnd,
        material: Handle<GndMaterial>,
        world: &mut World,
        load_context: &mut LoadContext,
    ) {
        let mut vertices = vec![];
        let mut uvs = vec![];
        let mut texture_ids = vec![];
        for (i, cube) in gnd.ground_mesh_cubes.iter().enumerate() {
            if cube.upwards_facing_surface >= 0 {
                Self::build_up_face(
                    gnd,
                    cube,
                    &mut vertices,
                    &mut uvs,
                    &mut texture_ids,
                    i as i32,
                );
            };
            if cube.east_facing_surface >= 0 {
                Self::build_east_face(
                    gnd,
                    cube,
                    &gnd.ground_mesh_cubes[i + 1],
                    &mut vertices,
                    &mut uvs,
                    &mut texture_ids,
                    i as i32,
                );
            };
            if cube.north_facing_surface >= 0 {
                Self::build_north_face(
                    gnd,
                    cube,
                    &gnd.ground_mesh_cubes[i + gnd.width as usize],
                    &mut vertices,
                    &mut uvs,
                    &mut texture_ids,
                    i as i32,
                );
            };
        }

        let asset_usage = if cfg!(feature = "debug") {
            RenderAssetUsages::all()
        } else {
            RenderAssetUsages::RENDER_WORLD
        };

        let mesh = load_context.add_labeled_asset(
            "Ground".to_string(),
            Mesh::new(PrimitiveTopology::TriangleList, asset_usage)
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                .with_inserted_attribute(GndMaterial::TEXTURE_ID_VERTEX_ATTRIBUTE, texture_ids)
                .with_computed_flat_normals(),
        );

        world.spawn((
            Name::new("Ground"),
            Ground,
            Mesh3d(mesh),
            MeshMaterial3d(material),
        ));
    }

    fn build_up_face(
        gnd: &gnd::Gnd,
        cube: &gnd::GroundMeshCube,
        vertices: &mut Vec<Vec3>,
        uvs: &mut Vec<Vec2>,
        texture_ids: &mut Vec<u32>,
        i: i32,
    ) {
        let surface = &gnd.surfaces[cube.upwards_facing_surface as usize];

        // Referent to the current cube
        let bottom_left_x = ((i % gnd.width as i32) - (gnd.width as i32 / 2)) as f32 * gnd.scale;
        let bottom_left_z = ((i / gnd.width as i32) - (gnd.height as i32 / 2)) as f32 * gnd.scale;

        vertices.push(Vec3::new(
            bottom_left_x,
            cube.bottom_left_height,
            bottom_left_z,
        ));
        vertices.push(Vec3::new(
            bottom_left_x + gnd.scale,
            cube.bottom_right_height,
            bottom_left_z,
        ));
        vertices.push(Vec3::new(
            bottom_left_x,
            cube.top_left_height,
            bottom_left_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_left));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_left));
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);

        vertices.push(Vec3::new(
            bottom_left_x + gnd.scale,
            cube.bottom_right_height,
            bottom_left_z,
        ));
        vertices.push(Vec3::new(
            bottom_left_x + gnd.scale,
            cube.top_right_height,
            bottom_left_z + gnd.scale,
        ));
        vertices.push(Vec3::new(
            bottom_left_x,
            cube.top_left_height,
            bottom_left_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_right));
        uvs.push(Vec2::from_array(surface.top_left));
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
    }

    fn build_east_face(
        gnd: &gnd::Gnd,
        cube: &gnd::GroundMeshCube,
        east_cube: &gnd::GroundMeshCube,
        vertices: &mut Vec<Vec3>,
        uvs: &mut Vec<Vec2>,
        texture_ids: &mut Vec<u32>,
        i: i32,
    ) {
        let surface = &gnd.surfaces[cube.east_facing_surface as usize];

        // Referent to the current cube
        let bottom_right_x =
            (((i + 1) % gnd.width as i32) - (gnd.width as i32 / 2)) as f32 * gnd.scale;
        let bottom_right_z = ((i / gnd.width as i32) - (gnd.height as i32 / 2)) as f32 * gnd.scale;

        vertices.push(Vec3::new(
            bottom_right_x,
            cube.bottom_right_height,
            bottom_right_z,
        ));
        vertices.push(Vec3::new(
            bottom_right_x,
            east_cube.bottom_left_height,
            bottom_right_z,
        ));
        vertices.push(Vec3::new(
            bottom_right_x,
            cube.top_right_height,
            bottom_right_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_right));
        uvs.push(Vec2::from_array(surface.bottom_left));
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);

        vertices.push(Vec3::new(
            bottom_right_x,
            cube.top_right_height,
            bottom_right_z + gnd.scale,
        ));
        vertices.push(Vec3::new(
            bottom_right_x,
            east_cube.bottom_left_height,
            bottom_right_z,
        ));
        vertices.push(Vec3::new(
            bottom_right_x,
            east_cube.top_left_height,
            bottom_right_z + gnd.scale,
        ));
        uvs.push(Vec2::from_array(surface.bottom_left));
        uvs.push(Vec2::from_array(surface.top_right));
        uvs.push(Vec2::from_array(surface.top_left));
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
    }

    fn build_north_face(
        gnd: &gnd::Gnd,
        cube: &gnd::GroundMeshCube,
        north_cube: &gnd::GroundMeshCube,
        vertices: &mut Vec<Vec3>,
        uvs: &mut Vec<Vec2>,
        texture_ids: &mut Vec<u32>,
        i: i32,
    ) {
        let surface = &gnd.surfaces[cube.north_facing_surface as usize];

        // Referent to the current cube
        let top_left_x = ((i % gnd.width as i32) - (gnd.width as i32 / 2)) as f32 * gnd.scale;
        let top_left_z = (((i + gnd.width as i32) / gnd.width as i32) - (gnd.height as i32 / 2))
            as f32
            * gnd.scale;

        vertices.push(Vec3::new(top_left_x, cube.top_left_height, top_left_z));
        vertices.push(Vec3::new(
            top_left_x + gnd.scale,
            cube.top_right_height,
            top_left_z,
        ));
        vertices.push(Vec3::new(
            top_left_x,
            north_cube.bottom_left_height,
            top_left_z,
        ));
        uvs.push(Vec2::from_array(surface.bottom_left));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_left));
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);

        vertices.push(Vec3::new(
            top_left_x,
            north_cube.bottom_left_height,
            top_left_z,
        ));
        vertices.push(Vec3::new(
            top_left_x + gnd.scale,
            cube.top_right_height,
            top_left_z,
        ));
        vertices.push(Vec3::new(
            top_left_x + gnd.scale,
            north_cube.bottom_right_height,
            top_left_z,
        ));
        uvs.push(Vec2::from_array(surface.top_left));
        uvs.push(Vec2::from_array(surface.bottom_right));
        uvs.push(Vec2::from_array(surface.top_right));
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
        texture_ids.push(surface.texture_id as u32);
    }

    #[must_use]
    fn build_water_plane(
        gnd: &gnd::Gnd,
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
        let material: [Handle<water_plane::WaterPlaneMaterial>; 32] =
            std::array::from_fn(|i| Self::water_plane_material(load_context, water_plane, name, i));

        world
            .spawn((
                Name::new(name.to_owned()),
                Transform::default(),
                Visibility::default(),
                Mesh3d(mesh),
                water_plane::WaterPlane::new(material, water_plane.texture_cyclical_interval),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id()
    }

    async fn build_ground_texture_atlas(
        load_context: &mut LoadContext<'_>,
        texture_paths: &[Box<str>],
    ) -> Handle<GndMaterial> {
        let mut images = Vec::with_capacity(texture_paths.len());

        for path in texture_paths.iter() {
            let direct_image = load_context
                .loader()
                .immediate()
                .load::<Image>(format!("{}{}", paths::TEXTURE_FILES_FOLDER, path))
                .await;
            match direct_image {
                Ok(image) => {
                    images.push(image.take());
                }
                Err(err) => {
                    bevy::log::error!("Could not load {} with error '{:?}'", path, err);
                    images.push(Image::new_fill(
                        Extent3d {
                            width: 8,
                            height: 8,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        &[255, 0, 255, 255],
                        TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::RENDER_WORLD,
                    ));
                }
            }
        }

        let (color_texture_image, texture_uvs) =
            helper::build_texture_atlas_from_list_of_images(&images, TextureFormat::Rgba8UnormSrgb);

        let storage_buffer = load_context.add_labeled_asset(
            "TextureAtlas/UVs".to_string(),
            ShaderStorageBuffer::new(
                texture_uvs
                    .iter()
                    .flat_map(Vec2::to_array)
                    .flat_map(f32::to_le_bytes)
                    .collect::<Vec<_>>()
                    .as_slice(),
                RenderAssetUsages::RENDER_WORLD,
            ),
        );

        let color_texture =
            load_context.add_labeled_asset("TextureAtlas/Image".to_string(), color_texture_image);
        load_context.add_labeled_asset(
            "TextureAtlas".to_string(),
            GndMaterial {
                color_texture,
                texture_uvs: storage_buffer,
            },
        )
    }

    #[must_use]
    fn water_plane_mesh(gnd: &gnd::Gnd, water_plane: &common::WaterPlane) -> Mesh {
        let mut vertices: Vec<Vec3> = vec![];
        let mut normals: Vec<Vec3> = vec![];
        let mut uvs: Vec<Vec2> = vec![];
        let mut indices: Vec<u16> = vec![];

        let mut insert_vertex = |x: u32, z: u32| {
            let vertex_x = (x as i32 - ((gnd.width / 2) as i32)) as f32 * gnd.scale;
            let vertex_z = (z as i32 - ((gnd.height / 2) as i32)) as f32 * gnd.scale;
            vertices.push(Vec3::new(vertex_x, water_plane.water_level, vertex_z));
            normals.push(Vec3::NEG_Y);
            uvs.push(Vec2::new(x as f32, z as f32));
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
                    // Ragnarok's weird coordinate system caused this
                    .any(|height| height >= &(water_plane.water_level - water_plane.wave_height))
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
        water_plane: &common::WaterPlane,
        name: &str,
        frame: usize,
    ) -> Handle<water_plane::WaterPlaneMaterial> {
        load_context.labeled_asset_scope(
            format!("{}Material/Frame{}", name, frame),
            |load_context| {
                let image: Handle<Image> = load_context
                    .loader()
                    .with_settings(|m: &mut ImageLoaderSettings| {
                        m.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                            label: Some("WaterSampler".to_owned()),
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
                        water_plane.water_type,
                        frame
                    ));
                water_plane::WaterPlaneMaterial {
                    texture: image,
                    wave: water_plane::Wave {
                        wave_height: water_plane.wave_height,
                        wave_speed: water_plane.wave_speed,
                        wave_pitch: water_plane.wave_pitch.to_radians(),
                    },
                    opaque: water_plane.water_type == 4 || water_plane.water_type == 6,
                }
            },
        )
    }
}
