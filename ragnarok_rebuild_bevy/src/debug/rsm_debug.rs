use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle},
    color::{self, Color},
    math::Vec3,
    prelude::{
        Children, Entity, Gizmos, GlobalTransform, HierarchyQueryExt, IntoSystemConfigs, Mesh,
        Query, ReflectResource, Res, Resource, With,
    },
    reflect::Reflect,
    render::mesh::VertexAttributeValues,
};

use crate::assets::rsm;

const NORMAL_GIZMOS_LENGHT: f32 = 0.5;

pub struct RsmDebugPlugin;

impl Plugin for RsmDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Resources
            .register_type::<RsmDebug>()
            .init_resource::<RsmDebug>()
            // Systems
            .add_systems(
                Update,
                (
                    show_rsm_vertex_normal.run_if(show_rsm_vertex_normal_condition),
                    show_rsm_edges.run_if(show_rsm_edges_condition),
                ),
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
    models: Query<Entity, With<rsm::Model>>,
    children: Query<&Children>,
    global_transforms: Query<&GlobalTransform>,
    model_primitives: Query<&Handle<Mesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    for entity in models.iter() {
        for child in children.iter_descendants(entity) {
            let Ok(child_global_transform) = global_transforms.get(child) else {
                continue;
            };
            let Ok(mesh_handle) = model_primitives.get(child) else {
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

            let global_transform = child_global_transform;
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

fn show_rsm_vertex_normal_condition(show_rsm_vertex_normal: Res<RsmDebug>) -> bool {
    show_rsm_vertex_normal.show_normals
}

fn show_rsm_edges(
    mut gizmos: Gizmos,
    models: Query<Entity, With<rsm::Model>>,
    children: Query<&Children>,
    global_transforms: Query<&GlobalTransform>,
    model_primitives: Query<&Handle<Mesh>>,
    meshes: Res<Assets<Mesh>>,
) {
    for entity in models.iter() {
        for child in children.iter_descendants(entity) {
            let Ok(child_global_transform) = global_transforms.get(child) else {
                continue;
            };
            let Ok(mesh_handle) = model_primitives.get(child) else {
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

            let global_transform = child_global_transform;
            for v in vertex.chunks(3).map(|triad| {
                triad
                    .iter()
                    .map(|vertex| Vec3::from_array(*vertex))
                    .collect::<Vec<_>>()
            }) {
                gizmos.line(
                    global_transform.transform_point(v[0]),
                    global_transform.transform_point(v[1]),
                    color::palettes::css::ORANGE,
                );
                gizmos.line(
                    global_transform.transform_point(v[0]),
                    global_transform.transform_point(v[2]),
                    color::palettes::css::ORANGE,
                );
                gizmos.line(
                    global_transform.transform_point(v[1]),
                    global_transform.transform_point(v[2]),
                    color::palettes::css::ORANGE,
                );
            }
        }
    }
}

fn show_rsm_edges_condition(rsm_debug: Res<RsmDebug>) -> bool {
    rsm_debug.show_edges
}
