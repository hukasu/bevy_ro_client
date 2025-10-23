use std::borrow::Cow;

use bevy_asset::{Handle, LoadContext, RenderAssetUsages, io::Reader};
use bevy_camera::{primitives::Aabb, visibility::Visibility};
use bevy_ecs::{bundle::Bundle, entity::Entity, hierarchy::ChildOf, name::Name, world::World};
use bevy_image::Image;
use bevy_log::trace;
use bevy_math::Vec3;
use bevy_mesh::{Mesh, Mesh3d, MeshTag};
use bevy_pbr::MeshMaterial3d;
use bevy_ragnarok_water_plane::{WaterPlaneAsset, WaterPlaneBuilder};
use bevy_render::storage::ShaderStorageBuffer;
use bevy_scene::Scene;
use bevy_transform::components::Transform;

use ragnarok_gnd::{Error, Gnd};

use crate::{
    Ground,
    assets::{GndAsset, GndCubeMaterials},
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
        let surfaces = Self::build_surfaces(&gnd, load_context);
        let materials = Self::build_materials(&gnd, &textures, surfaces.clone(), load_context);
        let scene = Self::build_scene(&gnd, &materials, load_context);

        Ok(GndAsset {
            scene,
            textures,
            surfaces,
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

    fn build_surfaces(
        gnd: &Gnd,
        load_context: &mut LoadContext<'_>,
    ) -> Handle<ShaderStorageBuffer> {
        let mut surfaces = Vec::with_capacity(gnd.surfaces.len() * 8 * 4);

        for surface in &gnd.surfaces {
            surfaces.extend_from_slice(&surface.bottom_left[0].to_le_bytes());
            surfaces.extend_from_slice(&surface.bottom_left[1].to_le_bytes());
            surfaces.extend_from_slice(&surface.bottom_right[0].to_le_bytes());
            surfaces.extend_from_slice(&surface.bottom_right[1].to_le_bytes());
            surfaces.extend_from_slice(&surface.top_left[0].to_le_bytes());
            surfaces.extend_from_slice(&surface.top_left[1].to_le_bytes());
            surfaces.extend_from_slice(&surface.top_right[0].to_le_bytes());
            surfaces.extend_from_slice(&surface.top_right[1].to_le_bytes());
        }

        load_context.add_labeled_asset(
            "Surfaces".to_owned(),
            ShaderStorageBuffer::new(&surfaces, RenderAssetUsages::RENDER_WORLD),
        )
    }

    fn build_materials(
        gnd: &Gnd,
        textures: &[Handle<Image>],
        surfaces: Handle<ShaderStorageBuffer>,
        load_context: &mut LoadContext<'_>,
    ) -> Vec<GndCubeMaterials> {
        let Ok(width) = usize::try_from(gnd.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(gnd.height) else {
            unreachable!("Height must fit on usize");
        };

        let mut materials = Vec::with_capacity(width + height);

        for z in 0..height {
            for x in 0..width {
                let up_cube = &gnd.ground_mesh_cubes[x + z * width];

                let [up_material, east_material, north_material] = [
                    (
                        "Up",
                        gnd.get_top_face_heights(x, z),
                        usize::try_from(up_cube.upwards_facing_surface),
                    ),
                    (
                        "East",
                        gnd.get_east_face_heights(x, z),
                        usize::try_from(up_cube.east_facing_surface),
                    ),
                    (
                        "North",
                        gnd.get_north_face_heights(x, z),
                        usize::try_from(up_cube.north_facing_surface),
                    ),
                ]
                .map(|(discriminator, heights, surface_id)| {
                    if let Some(cube_heights) = heights
                        && let Ok(surface_id) = surface_id
                    {
                        let surface = &gnd.surfaces[surface_id];
                        let up = load_context.add_labeled_asset(
                            format!("Material{x}/{z}/{discriminator}"),
                            GndMaterial {
                                bottom_left: cube_heights[0],
                                bottom_right: cube_heights[1],
                                top_left: cube_heights[2],
                                top_right: cube_heights[3],
                                surface_id: u32::try_from(surface_id).unwrap_or(u32::MAX),
                                texture_id: u32::from(surface.texture_id),
                                surfaces: surfaces.clone(),
                                texture: textures[usize::from(surface.texture_id)].clone(),
                            },
                        );
                        Some(up)
                    } else {
                        None
                    }
                });

                materials.push(GndCubeMaterials {
                    up_material,
                    east_material,
                    north_material,
                });
            }
        }

        materials
    }

    fn build_scene(
        gnd: &Gnd,
        materials: &[GndCubeMaterials],
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

    fn build_cubes(world: &mut World, gnd: &Gnd, ground: Entity, materials: &[GndCubeMaterials]) {
        let Ok(width) = usize::try_from(gnd.width) else {
            unreachable!("Width must fit on usize");
        };
        let Ok(height) = usize::try_from(gnd.height) else {
            unreachable!("Height must fit on usize");
        };

        for i in 0..gnd.ground_mesh_cubes.len() {
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

            let cube_materials = &materials[x + z * width];

            for (i, (discriminator, mesh, material)) in [
                ("Up", &GND_TOP_MESH, &cube_materials.up_material),
                ("East", &GND_EAST_MESH, &cube_materials.east_material),
                ("North", &GND_NORTH_MESH, &cube_materials.north_material),
            ]
            .into_iter()
            .enumerate()
            {
                if let Some(material) = material {
                    let Ok(tag) = u32::try_from((x + z * width) * 3 + i) else {
                        unreachable!("Tag must fit in u32.");
                    };
                    world.spawn(Self::build_cube_face(
                        tag,
                        cube_entity,
                        discriminator,
                        mesh.clone(),
                        aabb,
                        material.clone(),
                    ));
                }
            }
        }
    }

    fn build_cube_face(
        tag: u32,
        cube_entity: Entity,
        discriminator: &'static str,
        mesh: Handle<Mesh>,
        aabb: Aabb,
        material: Handle<GndMaterial>,
    ) -> impl Bundle {
        (
            Name::new(format!("{discriminator} Face")),
            aabb,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            MeshTag(tag),
            Transform::default(),
            Visibility::default(),
            ChildOf(cube_entity),
        )
    }
}
