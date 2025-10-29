use bevy_app::{PostUpdate, Update};
use bevy_asset::Assets;
use bevy_camera::visibility::VisibilitySystems;
use bevy_color::{self, Color, Srgba, palettes};
use bevy_ecs::{
    entity::Entity,
    event::{EntityEvent, Event},
    hierarchy::Children,
    lifecycle::Add,
    observer::On,
    query::{With, Without},
    reflect::ReflectResource,
    resource::Resource,
    schedule::{
        IntoScheduleConfigs,
        common_conditions::{not, resource_changed},
    },
    system::{Commands, Local, Populated, Query, Res, ResMut, Single, SystemParam},
};
use bevy_gizmos::{GizmoAsset, aabb::ShowAabbGizmo, retained::Gizmo};
use bevy_log::debug;
use bevy_math::Vec3;
use bevy_mesh::{Mesh, Mesh3d, MeshTag};
use bevy_pbr::MeshMaterial3d;
use bevy_reflect::Reflect;
use bevy_render::storage::ShaderStorageBuffer;
use bevy_transform::TransformSystems;

use crate::{Cube, material::GndMaterial};

const AABB_COLOR: Srgba = palettes::tailwind::PURPLE_300;

pub(crate) struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Resources
        app.register_type::<GndDebug>().init_resource::<GndDebug>();
        app.add_systems(
            Update,
            trigger_on_changes.run_if(resource_changed::<GndDebug>),
        );
        app.add_systems(
            PostUpdate,
            (
                enable_gnd_edges.run_if(enable_gnd_edges_condition),
                disable_gnd_edges.run_if(not(enable_gnd_edges_condition)),
                enable_gnd_normals.run_if(enable_gnd_normals_condition),
                disable_gnd_normals.run_if(not(enable_gnd_normals_condition)),
            )
                .after(TransformSystems::Propagate)
                .after(VisibilitySystems::CheckVisibility),
        );
        // Observers
        app.add_observer(toggle_gnd_aabbs);
        app.add_observer(toggle_gnd_edges);
        app.add_observer(toggle_gnd_normals);
        app.add_observer(enable_gnd_aabbs_for_new_cubes);
    }
}

#[derive(Debug, Clone, Copy, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GndDebug {
    show_aabbs: bool,
    show_edges: bool,
    show_normals: bool,
}

#[derive(Debug, Event)]
pub struct ToggleGndAabbs;

#[derive(Debug, Event)]
pub struct ToggleGndEdges;

#[derive(Debug, Event)]
pub struct ToggleGndNormals;

#[derive(SystemParam)]
struct GndAssets<'w> {
    meshes: Res<'w, Assets<Mesh>>,
    gnd_materials: Res<'w, Assets<GndMaterial>>,
    shader_buffers: Res<'w, Assets<ShaderStorageBuffer>>,
}

fn toggle_gnd_aabbs(_event: On<ToggleGndAabbs>, mut gat_debug: ResMut<GndDebug>) {
    debug!("Toggling Gnd Aabbs");
    gat_debug.show_aabbs = !gat_debug.show_aabbs;
}

fn enable_gnd_aabbs_for_new_cubes(
    event: On<Add, Cube>,
    mut commands: Commands,
    cubes: Query<&Children, With<Cube>>,
    gnd_debug: ResMut<GndDebug>,
) {
    if gnd_debug.show_aabbs
        && let Ok(children) = cubes.get(event.event_target())
        && let Some(child) = children.first()
    {
        commands.entity(*child).insert(ShowAabbGizmo {
            color: Some(AABB_COLOR.into()),
        });
    }
}

fn enable_gnd_aabbs(mut commands: Commands, cubes: Query<&Children, With<Cube>>) {
    debug!("Enabling Gnd Aabbs");
    let cube_aabb_color = AABB_COLOR.into();
    for children in cubes {
        if let Some(child) = children.first() {
            commands.entity(*child).insert(ShowAabbGizmo {
                color: Some(cube_aabb_color),
            });
        }
    }
}

fn disable_gnd_aabbs(mut commands: Commands, cubes: Query<&Children, With<Cube>>) {
    debug!("Disabling Gat Aabbs");
    for child in cubes.iter().flatten() {
        commands.entity(*child).remove::<ShowAabbGizmo>();
    }
}

fn toggle_gnd_edges(_event: On<ToggleGndEdges>, mut gat_debug: ResMut<GndDebug>) {
    debug!("Toggling Gnd Edges");
    gat_debug.show_edges = !gat_debug.show_edges;
}

fn enable_gnd_edges_condition(gnd_debug: Res<GndDebug>) -> bool {
    gnd_debug.show_edges
}

fn enable_gnd_edges(
    mut commands: Commands,
    faces: Populated<(Entity, &Mesh3d, &MeshMaterial3d<GndMaterial>, &MeshTag), Without<Gizmo>>,
    gnd_assets: GndAssets<'_>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    let edge_color: Color = palettes::tailwind::ORANGE_700.into();

    for (face, mesh, material, tag) in faces.into_inner() {
        let Some(vertices) = gnd_assets
            .meshes
            .get(mesh.id())
            .and_then(|mesh| mesh.attribute(Mesh::ATTRIBUTE_POSITION))
            .and_then(|vertices| vertices.as_float3())
        else {
            continue;
        };
        let Some(material) = gnd_assets.gnd_materials.get(material.id()) else {
            continue;
        };
        let Some(cube_heights) = gnd_assets
            .shader_buffers
            .get(material.cube_faces.id())
            .and_then(|buffer| buffer.data.as_ref())
        else {
            continue;
        };
        let Ok(tag) = usize::try_from(tag.0) else {
            unreachable!("Tag must fit in usize.");
        };

        let heights = cube_heights
            [(tag * GndMaterial::HEIGHTS_STRIDE)..((tag + 1) * GndMaterial::HEIGHTS_STRIDE)]
            .chunks(4)
            .map(|le| {
                let Ok(le) = le.try_into() else {
                    unreachable!("Array must always be length 4.");
                };
                f32::from_le_bytes(le)
            })
            .collect::<Vec<_>>();

        let mut gizmos = GizmoAsset::new();
        gizmos.line(
            Vec3::from_array(vertices[0]).with_y(heights[0]),
            Vec3::from_array(vertices[1]).with_y(heights[1]),
            edge_color,
        );
        gizmos.line(
            Vec3::from_array(vertices[0]).with_y(heights[0]),
            Vec3::from_array(vertices[2]).with_y(heights[2]),
            edge_color,
        );
        gizmos.line(
            Vec3::from_array(vertices[1]).with_y(heights[1]),
            Vec3::from_array(vertices[2]).with_y(heights[2]),
            edge_color,
        );
        gizmos.line(
            Vec3::from_array(vertices[1]).with_y(heights[1]),
            Vec3::from_array(vertices[3]).with_y(heights[3]),
            edge_color,
        );
        gizmos.line(
            Vec3::from_array(vertices[2]).with_y(heights[2]),
            Vec3::from_array(vertices[3]).with_y(heights[3]),
            edge_color,
        );

        let handle = gizmo_assets.add(gizmos);

        commands.entity(face).insert(Gizmo {
            handle,
            ..Default::default()
        });
    }
}

fn disable_gnd_edges(
    mut commands: Commands,
    faces: Populated<Entity, (With<MeshMaterial3d<GndMaterial>>, With<Gizmo>)>,
) {
    for face in faces.into_inner() {
        commands.entity(face).remove::<Gizmo>();
    }
}

fn toggle_gnd_normals(_event: On<ToggleGndNormals>, mut gat_debug: ResMut<GndDebug>) {
    debug!("Toggling Gnd Normals");
    gat_debug.show_normals = !gat_debug.show_normals;
}

fn enable_gnd_normals_condition(gnd_debug: Res<GndDebug>) -> bool {
    gnd_debug.show_normals
}

#[expect(clippy::type_complexity, reason = "Queries are complex")]
fn enable_gnd_normals(
    mut commands: Commands,
    cubes: Populated<(Entity, Option<&Children>), (With<Cube>, Without<Gizmo>)>,
    faces: Query<(&Mesh3d, &MeshMaterial3d<GndMaterial>, &MeshTag)>,
    gnd_assets: GndAssets<'_>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    for (cube, children) in cubes.into_inner() {
        let Some(cube_faces) = children else {
            continue;
        };
        let Some((mesh, material, tag)) = cube_faces
            .iter()
            .filter_map(|child| {
                faces
                    .get(*child)
                    .ok()
                    .filter(|(_, _, tag)| tag.0.is_multiple_of(3))
            })
            .next()
        else {
            continue;
        };

        let Some(vertices) = gnd_assets
            .meshes
            .get(mesh.id())
            .and_then(|mesh| mesh.attribute(Mesh::ATTRIBUTE_POSITION))
            .and_then(|vertices| vertices.as_float3())
        else {
            continue;
        };
        let Some(material) = gnd_assets.gnd_materials.get(material.id()) else {
            continue;
        };
        let Some(cube_heights) = gnd_assets
            .shader_buffers
            .get(material.cube_faces.id())
            .and_then(|buffer| buffer.data.as_ref())
        else {
            continue;
        };
        let Some(cube_normals) = gnd_assets
            .shader_buffers
            .get(material.normals.id())
            .and_then(|buffer| buffer.data.as_ref())
        else {
            continue;
        };
        let Ok(tag) = usize::try_from(tag.0) else {
            unreachable!("Tag must fit in usize.");
        };

        let heights = cube_heights
            [(tag * GndMaterial::HEIGHTS_STRIDE)..((tag + 1) * GndMaterial::HEIGHTS_STRIDE)]
            .chunks(4)
            .map(|le| {
                let Ok(le) = le.try_into() else {
                    unreachable!("Array must always be length 4.");
                };
                f32::from_le_bytes(le)
            })
            .collect::<Vec<_>>();
        let normals = cube_normals[((tag / 3) * GndMaterial::NORMALS_STRIDE)
            ..(((tag / 3) + 1) * GndMaterial::NORMALS_STRIDE)]
            .chunks(16)
            .map(|le| {
                let Ok(le): Result<[u8; 16], _> = le.try_into() else {
                    unreachable!("Array must always be length 4.");
                };
                let normal = Vec3::new(
                    f32::from_le_bytes([le[0], le[1], le[2], le[3]]),
                    f32::from_le_bytes([le[4], le[5], le[6], le[7]]),
                    f32::from_le_bytes([le[8], le[9], le[10], le[11]]),
                );
                debug_assert!(normal.is_normalized());
                normal
            })
            .collect::<Vec<_>>();

        let mut gizmos = GizmoAsset::new();
        for i in 0..4 {
            gizmos.arrow(
                Vec3::from_array(vertices[i]).with_y(heights[i]),
                Vec3::from_array(vertices[i]).with_y(heights[i])
                    + normals[i] * Vec3::new(1., 5., 1.),
                Color::srgb(normals[i].x.abs(), normals[i].y.abs(), normals[i].z.abs()),
            );
        }

        let handle = gizmo_assets.add(gizmos);

        commands.entity(cube).insert(Gizmo {
            handle,
            ..Default::default()
        });
    }
}

fn disable_gnd_normals(
    mut commands: Commands,
    cubes: Populated<Entity, (With<Cube>, With<Gizmo>)>,
) {
    for cube in cubes.into_inner() {
        commands.entity(cube).remove::<Gizmo>();
    }
}

fn trigger_on_changes(
    mut commands: Commands,
    gnd_debug: Res<GndDebug>,
    mut gnd_debug_cache: Local<GndDebug>,
) {
    if gnd_debug.show_aabbs != gnd_debug_cache.show_aabbs {
        match gnd_debug.show_aabbs {
            true => commands.run_system_cached(enable_gnd_aabbs),
            false => commands.run_system_cached(disable_gnd_aabbs),
        }
    }

    *gnd_debug_cache = *gnd_debug;
}
