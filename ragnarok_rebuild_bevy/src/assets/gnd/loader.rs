use bevy::{
    asset::{io::Reader, Handle, LoadContext, RenderAssetUsages},
    ecs::world::World,
    image::Image,
    math::{Vec2, Vec3},
    mesh::{Mesh, PrimitiveTopology},
    pbr::MeshMaterial3d,
    prelude::{Mesh3d, Name},
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        storage::ShaderStorageBuffer,
    },
    scene::Scene,
};

use ragnarok_rebuild_assets::gnd;

use ragnarok_water_plane::WaterPlane;
use serde::{Deserialize, Serialize};

use crate::{assets::paths, helper};

use super::{components::Ground, material::GndMaterial, GroundScale};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetLoaderSettings {
    pub water_plane: Option<WaterPlane>,
}

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = AssetLoaderSettings;
    type Error = gnd::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        bevy::log::trace!("Loading Gnd {:?}.", load_context.path());

        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let gnd = gnd::Gnd::from_reader(&mut data.as_slice())?;

        Ok(Self::generate_ground(&gnd, load_context).await)
    }

    fn extensions(&self) -> &[&str] {
        &["gnd"]
    }
}

impl AssetLoader {
    async fn generate_ground(gnd: &gnd::Gnd, load_context: &mut LoadContext<'_>) -> Scene {
        let mut world = World::new();

        // 2x2 tiles per gnd cube
        world.insert_resource(GroundScale(2. / gnd.scale));
        Self::generate_ground_mesh(gnd, &mut world, load_context).await;

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
}
