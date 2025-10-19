use bevy_asset::{Handle, LoadContext, RenderAssetUsages, io::Reader};
use bevy_camera::visibility::Visibility;
use bevy_ecs::{name::Name, world::World};
use bevy_image::Image;
use bevy_log::{error, trace};
use bevy_math::{Vec2, Vec3};
use bevy_mesh::{Mesh, Mesh3d, PrimitiveTopology};
use bevy_pbr::MeshMaterial3d;
use bevy_render::{
    render_resource::{Extent3d, TextureDimension, TextureFormat},
    storage::ShaderStorageBuffer,
};
use bevy_scene::Scene;
use bevy_transform::components::Transform;

use ragnarok_gnd::{Error, Gnd, GroundMeshCube};
use ragnarok_rebuild_bevy::{assets::paths, helper};
use serde::{Deserialize, Serialize};

use ragnarok_water_plane::WaterPlane;

use crate::assets::GndAsset;

use super::{GroundScale, components::Ground, material::GndMaterial};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AssetLoaderSettings {
    pub water_plane: Option<WaterPlane>,
}

/// Asset loader for [`GndAsset`]
///
/// ## Labeled assets
///
/// * `Mesh`: [`Mesh`](bevy_mesh::Mesh) = Mesh built from the [`Gnd`]
///   cubes.
/// * `Material`: [`GndMaterial`] = Material generated from [`Gnd`]
///   textures.
/// * `Scene`: [`Scene`](bevy_scene::Scene) = Scene containing all objects represented
///   by the [`Gnd`].
pub struct AssetLoader;

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = GndAsset;
    type Settings = AssetLoaderSettings;
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        trace!("Loading Gnd {:?}.", load_context.path());

        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let gnd = Gnd::from_reader(&mut data.as_slice())?;

        let mesh = Self::build_cubes(&gnd, load_context);
        let material = Self::build_ground_texture_atlas(&gnd, load_context).await;
        let scene =
            Self::build_scene(&gnd, mesh.clone(), material.clone(), settings, load_context).await;

        Ok(GndAsset {
            mesh,
            material,
            scene,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gnd"]
    }
}

impl AssetLoader {
    async fn build_scene(
        gnd: &Gnd,
        mesh: Handle<Mesh>,
        material: Handle<GndMaterial>,
        _settings: &AssetLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Handle<Scene> {
        let mut world = World::new();

        // 2x2 tiles per gnd cube
        world.insert_resource(GroundScale(2. / gnd.scale));

        world.spawn((
            Name::new("Ground".to_owned()),
            Ground,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::default(),
        ));

        load_context.add_labeled_asset("Scene".to_owned(), Scene::new(world))
    }

    fn build_cubes(gnd: &Gnd, load_context: &mut LoadContext) -> Handle<Mesh> {
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

        load_context.add_labeled_asset(
            "Ground".to_string(),
            Mesh::new(PrimitiveTopology::TriangleList, asset_usage)
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                .with_inserted_attribute(GndMaterial::TEXTURE_ID_VERTEX_ATTRIBUTE, texture_ids)
                .with_computed_flat_normals(),
        )
    }

    fn build_up_face(
        gnd: &Gnd,
        cube: &GroundMeshCube,
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
        gnd: &Gnd,
        cube: &GroundMeshCube,
        east_cube: &GroundMeshCube,
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
        gnd: &Gnd,
        cube: &GroundMeshCube,
        north_cube: &GroundMeshCube,
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
        gnd: &Gnd,
        load_context: &mut LoadContext<'_>,
    ) -> Handle<GndMaterial> {
        let texture_paths = gnd.textures.as_ref();
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
                    error!("Could not load {} with error '{:?}'", path, err);
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
