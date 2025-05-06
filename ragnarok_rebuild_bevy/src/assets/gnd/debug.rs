use bevy::{
    app::PostUpdate,
    asset::Assets,
    color::{self, Color, Srgba},
    math::Vec3,
    prelude::{
        Gizmos, GlobalTransform, IntoScheduleConfigs, Mesh, Mesh3d, Query, ReflectResource, Res,
        Resource, ViewVisibility, With,
    },
    reflect::Reflect,
    render::{
        mesh::{Indices, VertexAttributeValues},
        view::VisibilitySystems,
    },
};

use crate::assets::gnd;

const NORMAL_GIZMOS_LENGHT: f32 = 0.5;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Resources
            .register_type::<GndDebug>()
            .init_resource::<GndDebug>()
            // Systems
            .add_systems(
                PostUpdate,
                (
                    show_gnd_vertex_normal.run_if(show_gnd_vertex_normal_condition),
                    show_gnd_edges.run_if(show_gnd_edges_condition),
                )
                    .after(VisibilitySystems::CheckVisibility),
            );
    }
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GndDebug {
    show_edges: bool,
    show_normals: bool,
}

fn show_gnd_vertex_normal(
    mut gizmos: Gizmos,
    grounds: Query<(&GlobalTransform, &Mesh3d, &ViewVisibility), With<gnd::Ground>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (ground_transform, ground_mesh, ground_in_view) in grounds.iter() {
        if !**ground_in_view {
            continue;
        }

        let Some(mesh) = meshes.get(ground_mesh) else {
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
                let start = ground_transform.transform_point(vertex);
                let direction =
                    (ground_transform.transform_point(vertex + normal) - start).normalize();
                let color = Color::srgb_from_array(direction.to_array());
                gizmos.line(start, start + (direction * NORMAL_GIZMOS_LENGHT), color);
            }
        } else {
            for (v, n) in vertex.iter().zip(normals) {
                let vertex = Vec3::from_array(*v);
                let normal = Vec3::from_array(*n);
                let start = ground_transform.transform_point(vertex);
                let direction =
                    (ground_transform.transform_point(vertex + normal) - start).normalize();
                let color = Color::srgb_from_array(direction.to_array());
                gizmos.line(start, start + (direction * NORMAL_GIZMOS_LENGHT), color);
            }
        }
    }
}

fn show_gnd_vertex_normal_condition(gnd_debug: Res<GndDebug>) -> bool {
    gnd_debug.show_normals
}

fn show_gnd_edges(
    mut gizmos: Gizmos,
    grounds: Query<(&GlobalTransform, &Mesh3d, &ViewVisibility), With<gnd::Ground>>,
    meshes: Res<Assets<Mesh>>,
) {
    const GIZMO_COLOR: Srgba = color::palettes::css::ORANGE;
    for (ground_transform, ground_mesh, ground_in_view) in grounds.iter() {
        if !**ground_in_view {
            continue;
        }

        let Some(mesh) = meshes.get(ground_mesh) else {
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
                    ground_transform.transform_point(vertex[usize::from(i[0])]),
                    ground_transform.transform_point(vertex[usize::from(i[1])]),
                    GIZMO_COLOR,
                );
                gizmos.line(
                    ground_transform.transform_point(vertex[usize::from(i[0])]),
                    ground_transform.transform_point(vertex[usize::from(i[2])]),
                    GIZMO_COLOR,
                );
                gizmos.line(
                    ground_transform.transform_point(vertex[usize::from(i[1])]),
                    ground_transform.transform_point(vertex[usize::from(i[2])]),
                    GIZMO_COLOR,
                );
            }
        } else {
            for v in vertex.chunks(3) {
                gizmos.line(
                    ground_transform.transform_point(v[0]),
                    ground_transform.transform_point(v[1]),
                    GIZMO_COLOR,
                );
                gizmos.line(
                    ground_transform.transform_point(v[0]),
                    ground_transform.transform_point(v[2]),
                    GIZMO_COLOR,
                );
                gizmos.line(
                    ground_transform.transform_point(v[1]),
                    ground_transform.transform_point(v[2]),
                    GIZMO_COLOR,
                );
            }
        }
    }
}

fn show_gnd_edges_condition(gnd_debug: Res<GndDebug>) -> bool {
    gnd_debug.show_edges
}
