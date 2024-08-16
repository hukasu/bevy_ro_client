use bevy::{
    app::{Plugin, Update},
    asset::{Assets, Handle},
    color::Color,
    math::Vec3,
    prelude::{
        Children, Deref, Entity, Gizmos, GlobalTransform, HierarchyQueryExt, IntoSystemConfigs,
        Mesh, Query, ReflectResource, Res, Resource, With,
    },
    reflect::Reflect,
    render::mesh::VertexAttributeValues,
};

use crate::assets::rsm;

const NORMAL_GIZMOS_LENGHT: f32 = 5.;

pub struct ShowRsmVertexNormalsPlugin;

impl Plugin for ShowRsmVertexNormalsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Resources
            .register_type::<ShowRsmVertexNormals>()
            .init_resource::<ShowRsmVertexNormals>()
            // Systems
            .add_systems(
                Update,
                show_rsm_vertex_normal.run_if(show_rsm_vertex_normal_condition),
            );
    }
}

#[derive(Debug, Clone, Default, Resource, Deref, Reflect)]
#[reflect(Resource)]
pub struct ShowRsmVertexNormals(bool);

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
                let end =
                    global_transform.transform_point(vertex + (normal * NORMAL_GIZMOS_LENGHT));
                let color = Color::srgb_from_array((start - end).normalize().to_array());
                gizmos.line(start, end, color);
            }
        }
    }
}

fn show_rsm_vertex_normal_condition(show_rsm_vertex_normal: Res<ShowRsmVertexNormals>) -> bool {
    **show_rsm_vertex_normal
}
