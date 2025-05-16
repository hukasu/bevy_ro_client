use bevy_app::PostUpdate;
use bevy_asset::Assets;
use bevy_color::{Color, palettes};
use bevy_ecs::{
    entity::Entity,
    hierarchy::Children,
    query::With,
    reflect::ReflectResource,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Query, Res},
};
use bevy_gizmos::gizmos::Gizmos;
use bevy_math::Vec3;
use bevy_reflect::Reflect;
use bevy_render::{
    mesh::{Indices, Mesh, Mesh3d, VertexAttributeValues},
    view::{ViewVisibility, VisibilitySystems},
};
use bevy_transform::components::GlobalTransform;

const NORMAL_GIZMOS_LENGHT: f32 = 0.5;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Resources
            .register_type::<RsmDebug>()
            .init_resource::<RsmDebug>()
            // Systems
            .add_systems(
                PostUpdate,
                (
                    show_rsm_vertex_normal.run_if(show_rsm_vertex_normal_condition),
                    show_rsm_edges.run_if(show_rsm_edges_condition),
                )
                    .after(VisibilitySystems::CheckVisibility),
            );
    }
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct RsmDebug {
    show_edges: bool,
    show_normals: bool,
}

fn show_rsm_vertex_normal(
    mut gizmos: Gizmos,
    models: Query<Entity, With<crate::components::Model>>,
    children: Query<&Children>,
    global_transforms: Query<&GlobalTransform>,
    model_primitives: Query<(&Mesh3d, &ViewVisibility)>,
    meshes: Res<Assets<Mesh>>,
) {
    for entity in models.iter() {
        for child in children.iter_descendants(entity) {
            let Ok((mesh_handle, view_visibility)) = model_primitives.get(child) else {
                continue;
            };
            if !**view_visibility {
                continue;
            }
            let Ok(global_transform) = global_transforms.get(child) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_handle) else {
                continue;
            };
            let Some(VertexAttributeValues::Float32x3(vertex)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                continue;
            };
            let Some(VertexAttributeValues::Float32x3(normals)) =
                mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            else {
                continue;
            };

            if let Some(Indices::U16(indices)) = mesh.indices() {
                for i in indices {
                    let v = vertex[usize::from(*i)];
                    let n = normals[usize::from(*i)];
                    let vertex = Vec3::from_array(v);
                    let normal = Vec3::from_array(n);
                    let start = global_transform.transform_point(vertex);
                    let direction =
                        (global_transform.transform_point(vertex + normal) - start).normalize();
                    let color = Color::srgb_from_array(direction.to_array());
                    gizmos.line(start, start + (direction * NORMAL_GIZMOS_LENGHT), color);
                }
            } else {
                for (v, n) in vertex.iter().zip(normals) {
                    let vertex = Vec3::from_array(*v);
                    let normal = Vec3::from_array(*n);
                    let start = global_transform.transform_point(vertex);
                    let direction =
                        (global_transform.transform_point(vertex + normal) - start).normalize();
                    let color = Color::srgb_from_array(direction.to_array());
                    gizmos.line(start, start + (direction * NORMAL_GIZMOS_LENGHT), color);
                }
            }
        }
    }
}

fn show_rsm_vertex_normal_condition(show_rsm_vertex_normal: Res<RsmDebug>) -> bool {
    show_rsm_vertex_normal.show_normals
}

fn show_rsm_edges(
    mut gizmos: Gizmos,
    models: Query<Entity, With<crate::components::Model>>,
    children: Query<&Children>,
    global_transforms: Query<&GlobalTransform>,
    model_primitives: Query<(&Mesh3d, &ViewVisibility)>,
    meshes: Res<Assets<Mesh>>,
) {
    for entity in models.iter() {
        for child in children.iter_descendants(entity) {
            let Ok((mesh_handle, view_visibility)) = model_primitives.get(child) else {
                continue;
            };
            if !**view_visibility {
                continue;
            }
            let Ok(global_transform) = global_transforms.get(child) else {
                continue;
            };
            let Some(mesh) = meshes.get(mesh_handle) else {
                continue;
            };
            let Some(VertexAttributeValues::Float32x3(vertex)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            else {
                continue;
            };
            let vertex = vertex
                .iter()
                .map(|triangle| Vec3::from_array(*triangle))
                .collect::<Vec<_>>();

            if let Some(Indices::U16(indices)) = mesh.indices() {
                for i in indices.chunks(3) {
                    gizmos.line(
                        global_transform.transform_point(vertex[usize::from(i[0])]),
                        global_transform.transform_point(vertex[usize::from(i[1])]),
                        palettes::css::ORANGE,
                    );
                    gizmos.line(
                        global_transform.transform_point(vertex[usize::from(i[0])]),
                        global_transform.transform_point(vertex[usize::from(i[2])]),
                        palettes::css::ORANGE,
                    );
                    gizmos.line(
                        global_transform.transform_point(vertex[usize::from(i[1])]),
                        global_transform.transform_point(vertex[usize::from(i[2])]),
                        palettes::css::ORANGE,
                    );
                }
            } else {
                for v in vertex.chunks(3) {
                    gizmos.line(
                        global_transform.transform_point(v[0]),
                        global_transform.transform_point(v[1]),
                        palettes::css::ORANGE,
                    );
                    gizmos.line(
                        global_transform.transform_point(v[0]),
                        global_transform.transform_point(v[2]),
                        palettes::css::ORANGE,
                    );
                    gizmos.line(
                        global_transform.transform_point(v[1]),
                        global_transform.transform_point(v[2]),
                        palettes::css::ORANGE,
                    );
                }
            }
        }
    }
}

fn show_rsm_edges_condition(rsm_debug: Res<RsmDebug>) -> bool {
    rsm_debug.show_edges
}
