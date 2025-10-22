use std::{borrow::Cow, collections::HashMap};

use bevy_asset::{Handle, LoadContext, io::Reader};
use bevy_camera::{primitives::Aabb, visibility::Visibility};
use bevy_ecs::{entity::Entity, hierarchy::ChildOf, name::Name, world::World};
use bevy_image::Image;
use bevy_log::trace;
use bevy_math::{USizeVec3, Vec2, Vec3};
use bevy_mesh::{Mesh, Mesh3d};
use bevy_pbr::MeshMaterial3d;
use bevy_ragnarok_water_plane::{WaterPlaneAsset, WaterPlaneBuilder};
use bevy_scene::Scene;
use bevy_transform::components::Transform;

use ragnarok_gnd::{Error, Gnd};

use crate::{
    Ground,
    assets::GndAsset,
    material::GndMaterial,
    plugin::{GND_EAST_MESH, GND_NORTH_MESH, GND_TOP_MESH},
};

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
pub struct AssetLoader {
    pub texture_prefix: Cow<'static, str>,
}

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = GndAsset;
    type Settings = ();
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        trace!("Loading Gnd {:?}.", load_context.path());

        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let gnd = Gnd::from_reader(&mut data.as_slice())?;

        let textures = self.load_textures(&gnd, load_context);
        let materials = Self::build_materials(&gnd, &textures, load_context);
        let scene = Self::build_scene(&gnd, &materials, load_context);

        Ok(GndAsset {
            scene,
            textures,
            materials,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["gnd"]
    }
}

impl AssetLoader {
    fn load_textures(&self, gnd: &Gnd, load_context: &mut LoadContext<'_>) -> Vec<Handle<Image>> {
        gnd.textures
            .iter()
            .map(|texture| load_context.load(format!("{}{}", self.texture_prefix, texture)))
            .collect()
    }

    fn build_materials(
        gnd: &Gnd,
        textures: &[Handle<Image>],
        load_context: &mut LoadContext<'_>,
    ) -> HashMap<USizeVec3, Handle<GndMaterial>> {
        let Ok(width) = usize::try_from(gnd.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(gnd.height) else {
            unreachable!("Height must fit on usize");
        };

        let mut materials = HashMap::new();

        for x in 0..width {
            for z in 0..height {
                let up_cube = &gnd.ground_mesh_cubes[x + z * width];

                if let Some(up_cube_heights) = gnd.get_top_face_heights(x, z)
                    && let Ok(up_surface_id) = usize::try_from(up_cube.upwards_facing_surface)
                {
                    let up_surface = &gnd.surfaces[up_surface_id];
                    let up = load_context.add_labeled_asset(
                        format!("Material{x}/{z}/Up"),
                        GndMaterial {
                            bottom_left: up_cube_heights[0],
                            bottom_right: up_cube_heights[1],
                            top_left: up_cube_heights[2],
                            top_right: up_cube_heights[3],
                            bottom_left_uv: Vec2::from_array(up_surface.bottom_left),
                            bottom_right_uv: Vec2::from_array(up_surface.bottom_right),
                            top_left_uv: Vec2::from_array(up_surface.top_left),
                            top_right_uv: Vec2::from_array(up_surface.top_right),
                            texture: textures[usize::from(up_surface.texture_id)].clone(),
                        },
                    );
                    materials.insert(USizeVec3::new(x, z, up_surface_id), up);
                }

                if let Some(east_cube_heights) = gnd.get_east_face_heights(x, z)
                    && let Ok(east_surface_id) = usize::try_from(up_cube.east_facing_surface)
                {
                    let east_surface = &gnd.surfaces[east_surface_id];
                    let east = load_context.add_labeled_asset(
                        format!("Material{x}/{z}/East"),
                        GndMaterial {
                            bottom_left: east_cube_heights[0],
                            bottom_right: east_cube_heights[1],
                            top_left: east_cube_heights[2],
                            top_right: east_cube_heights[3],
                            bottom_left_uv: Vec2::from_array(east_surface.bottom_left),
                            bottom_right_uv: Vec2::from_array(east_surface.bottom_right),
                            top_left_uv: Vec2::from_array(east_surface.top_left),
                            top_right_uv: Vec2::from_array(east_surface.top_right),
                            texture: textures[usize::from(east_surface.texture_id)].clone(),
                        },
                    );
                    materials.insert(USizeVec3::new(x, z, east_surface_id), east);
                }

                if let Some(north_cube_heights) = gnd.get_north_face_heights(x, z)
                    && let Ok(north_surface_id) = usize::try_from(up_cube.north_facing_surface)
                {
                    let north_surface = &gnd.surfaces[north_surface_id];
                    let north = load_context.add_labeled_asset(
                        format!("Material{x}/{z}/North"),
                        GndMaterial {
                            bottom_left: north_cube_heights[0],
                            bottom_right: north_cube_heights[1],
                            top_left: north_cube_heights[2],
                            top_right: north_cube_heights[3],
                            bottom_left_uv: Vec2::from_array(north_surface.bottom_left),
                            bottom_right_uv: Vec2::from_array(north_surface.bottom_right),
                            top_left_uv: Vec2::from_array(north_surface.top_left),
                            top_right_uv: Vec2::from_array(north_surface.top_right),
                            texture: textures[usize::from(north_surface.texture_id)].clone(),
                        },
                    );
                    materials.insert(USizeVec3::new(x, z, north_surface_id), north);
                }
            }
        }

        materials
    }

    fn build_scene(
        gnd: &Gnd,
        materials: &HashMap<USizeVec3, Handle<GndMaterial>>,
        load_context: &mut LoadContext<'_>,
    ) -> Handle<Scene> {
        let mut world = World::new();

        let ground = world
            .spawn((
                Name::new("Ground".to_owned()),
                Ground {
                    width: gnd.width,
                    height: gnd.height,
                    scale: gnd.scale,
                },
                Transform::from_scale(Vec3::new(gnd.scale, 1., gnd.scale)),
                Visibility::default(),
            ))
            .id();

        Self::build_cubes(&mut world, gnd, ground, materials);

        for (i, water_plane) in gnd.water_planes.iter().enumerate() {
            world.spawn((
                Name::new("WaterPlane".to_owned()),
                WaterPlaneBuilder {
                    width: gnd.width - 2,
                    height: gnd.height - 2,
                    water_plane: load_context.add_labeled_asset(
                        format!("WaterPlane{i}"),
                        WaterPlaneAsset::from(water_plane),
                    ),
                },
                Transform::from_translation(Vec3::new(0., 0., 0.)),
                Visibility::default(),
            ));
        }

        load_context.add_labeled_asset("Scene".to_owned(), Scene::new(world))
    }

    fn build_cubes(
        world: &mut World,
        gnd: &Gnd,
        ground: Entity,
        materials: &HashMap<USizeVec3, Handle<GndMaterial>>,
    ) {
        let Ok(width) = usize::try_from(gnd.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(gnd.height) else {
            unreachable!("Height must fit on usize");
        };

        for (i, cube) in gnd.ground_mesh_cubes.iter().enumerate() {
            let x = i % width;
            let z = i / width;

            let tx = x as f32 - (width as f32) / 2. + 0.5;
            let tz = z as f32 - (height as f32) / 2. + 0.5;
            let transform = Transform::from_translation(Vec3::new(tx, 0., tz));

            let top_face = gnd.get_top_face_heights(x, z).map(|heights| {
                [
                    Vec3::new(-0.5, heights[0], -0.5),
                    Vec3::new(0.5, heights[1], -0.5),
                    Vec3::new(-0.5, heights[2], 0.5),
                    Vec3::new(0.5, heights[3], 0.5),
                ]
            });
            let east_face = gnd.get_east_face_heights(x, z).map(|heights| {
                [
                    Vec3::new(0.5, heights[0], -0.5),
                    Vec3::new(0.5, heights[1], -0.5),
                    Vec3::new(0.5, heights[2], 0.5),
                    Vec3::new(0.5, heights[3], 0.5),
                ]
            });
            let north_face = gnd.get_north_face_heights(x, z).map(|heights| {
                [
                    Vec3::new(-0.5, heights[0], 0.5),
                    Vec3::new(0.5, heights[1], 0.5),
                    Vec3::new(-0.5, heights[2], 0.5),
                    Vec3::new(0.5, heights[3], 0.5),
                ]
            });
            let Some(aabb) =
                Aabb::enclosing([top_face, east_face, north_face].iter().flatten().flatten())
            else {
                unreachable!("Should never be empty.");
            };

            let cube_entity = world
                .spawn((
                    Name::new(format!("Cube {x}/{z}")),
                    transform,
                    Visibility::default(),
                    ChildOf(ground),
                ))
                .id();

            Self::build_cube_face(
                world,
                cube_entity,
                "Up",
                GND_TOP_MESH.clone(),
                aabb,
                materials
                    .get(&USizeVec3::new(
                        x,
                        z,
                        usize::try_from(cube.upwards_facing_surface).unwrap_or(usize::MAX),
                    ))
                    .cloned(),
            );

            Self::build_cube_face(
                world,
                cube_entity,
                "East",
                GND_EAST_MESH.clone(),
                aabb,
                materials
                    .get(&USizeVec3::new(
                        x,
                        z,
                        usize::try_from(cube.east_facing_surface).unwrap_or(usize::MAX),
                    ))
                    .cloned(),
            );

            Self::build_cube_face(
                world,
                cube_entity,
                "North",
                GND_NORTH_MESH.clone(),
                aabb,
                materials
                    .get(&USizeVec3::new(
                        x,
                        z,
                        usize::try_from(cube.north_facing_surface).unwrap_or(usize::MAX),
                    ))
                    .cloned(),
            );
        }
    }

    fn build_cube_face(
        world: &mut World,
        cube_entity: Entity,
        discriminator: &'static str,
        mesh: Handle<Mesh>,
        aabb: Aabb,
        material: Option<Handle<GndMaterial>>,
    ) {
        if let Some(material) = material {
            world.spawn((
                Name::new(format!("{discriminator} Face")),
                aabb,
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
                Visibility::default(),
                ChildOf(cube_entity),
            ));
        }
    }
}
