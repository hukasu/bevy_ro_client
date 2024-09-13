use bevy::{
    asset::{io::Reader, AssetLoader as BevyAssetLoader, AsyncReadExt, Handle, LoadContext},
    core::Name,
    ecs::world::World,
    hierarchy::BuildWorldChildren,
    math::{Vec2, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    prelude::SpatialBundle,
    render::{
        mesh::{Mesh, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        texture::Image,
    },
    scene::Scene,
};
use ragnarok_rebuild_assets::gnd;

use crate::assets::paths;

use super::components::Ground;

pub struct AssetLoader;

impl BevyAssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = ();
    type Error = gnd::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            bevy::log::trace!("Loading Gnd {:?}.", load_context.path());
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let gnd = gnd::GND::from_reader(&mut data.as_slice())?;

            Ok(Self::generate_ground(&gnd, load_context))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gnd"]
    }
}

impl AssetLoader {
    fn generate_ground(gnd: &gnd::GND, load_context: &mut LoadContext) -> Scene {
        let textures: Vec<Handle<Image>> = gnd
            .textures
            .iter()
            .map(|path| load_context.load(format!("{}{}", paths::TEXTURE_FILES_FOLDER, path)))
            .collect();
        let materials: Vec<Handle<StandardMaterial>> = textures
            .iter()
            .cloned()
            .map(Self::mesh_material)
            .enumerate()
            .map(|(i, material)| load_context.add_labeled_asset(format!("Material{i}"), material))
            .collect();

        let mut world = World::new();

        Self::build_cubes(gnd, &materials, &mut world, load_context);

        Scene::new(world)
    }

    fn build_cubes(
        gnd: &gnd::GND,
        materials: &[Handle<StandardMaterial>],
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

        let meshs = groups
            .into_iter()
            .enumerate()
            .map(|(i, (vertex, uvs))| {
                load_context.add_labeled_asset(
                    format!("Primitive{i}"),
                    Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::RENDER_WORLD,
                    )
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertex)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                    .with_computed_flat_normals(),
                )
            })
            .collect::<Vec<_>>();

        let name: String = if let Some(filename) = load_context.path().file_name() {
            format!("{:?}", filename).trim_matches('"').to_owned()
        } else {
            "Unammed gnd".into()
        };

        world
            .spawn((Name::new(name), SpatialBundle::default(), Ground))
            .with_children(|parent| {
                for (i, mesh) in meshs.iter().enumerate() {
                    parent.spawn((
                        Name::new(format!("Primitive{i}")),
                        PbrBundle {
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
    fn mesh_material(texture: Handle<Image>) -> StandardMaterial {
        StandardMaterial {
            base_color_texture: Some(texture),
            reflectance: 0.2,
            ..Default::default()
        }
    }
}
