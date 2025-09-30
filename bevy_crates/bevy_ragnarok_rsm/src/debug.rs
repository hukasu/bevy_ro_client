use bevy_app::Update;
use bevy_asset::Assets;
use bevy_color::{Color, palettes};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::Event,
    hierarchy::{ChildOf, Children},
    observer::On,
    query::With,
    reflect::ReflectResource,
    resource::Resource,
    schedule::{IntoScheduleConfigs, common_conditions::resource_changed},
    system::{Commands, Local, Query, Res, ResMut},
};
use bevy_gizmos::{GizmoAsset, retained::Gizmo};
use bevy_math::Vec3;
use bevy_mesh::{Indices, Mesh, Mesh3d, VertexAttributeValues};
use bevy_platform::collections::HashMap;
use bevy_reflect::Reflect;
use bevy_transform::components::Transform;

const NORMAL_GIZMOS_LENGTH: f32 = 2.5;

pub(crate) struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Resources
            .register_type::<RsmDebug>()
            .init_resource::<RsmDebug>();

        // Systems
        app.add_systems(
            Update,
            trigger_on_changes.run_if(resource_changed::<RsmDebug>),
        );
        app.add_observer(toggle_rsm_edges);
        app.add_observer(toggle_rsm_normals);
    }
}

#[derive(Debug, Default, Clone, Copy, Reflect, Resource)]
#[reflect(Resource)]
pub struct RsmDebug {
    show_edges: bool,
    show_normals: bool,
}

/// Toggles gizmos for the Rsm model edges
#[derive(Debug, Event)]
pub struct ToggleRsmEdges;

/// Toggles gizmos for the Rsm model normals
#[derive(Debug, Event)]
pub struct ToggleRsmNormals;

/// Links to the edge [`Gizmo`]
#[derive(Debug, Component)]
#[relationship_target(relationship = EdgeGizmo)]
struct ShowingEdges(Entity);

/// Edge gizmo of a Rsm primitive
#[derive(Debug, Component)]
#[relationship(relationship_target = ShowingEdges)]
struct EdgeGizmo(Entity);

/// Links to the edge [`Gizmo`]
#[derive(Debug, Component)]
#[relationship_target(relationship = NormalGizmo)]
struct ShowingNormals(Entity);

/// Edge gizmo of a Rsm primitive
#[derive(Debug, Component)]
#[relationship(relationship_target = ShowingNormals)]
struct NormalGizmo(Entity);

fn toggle_rsm_edges(_event: On<ToggleRsmEdges>, mut rsm_debug: ResMut<RsmDebug>) {
    rsm_debug.show_edges = !rsm_debug.show_edges;
}

fn toggle_rsm_normals(_event: On<ToggleRsmNormals>, mut rsm_debug: ResMut<RsmDebug>) {
    rsm_debug.show_normals = !rsm_debug.show_normals;
}

fn trigger_on_changes(
    mut commands: Commands,
    rsm_debug: Res<RsmDebug>,
    mut rsm_debug_cache: Local<RsmDebug>,
) {
    if rsm_debug.show_edges != rsm_debug_cache.show_edges {
        match rsm_debug.show_edges {
            true => commands.run_system_cached(enable_rsm_edges),
            false => commands.run_system_cached(disable_rsm_edges),
        }
    }
    if rsm_debug.show_normals != rsm_debug_cache.show_normals {
        match rsm_debug.show_normals {
            true => commands.run_system_cached(enable_rsm_normals),
            false => commands.run_system_cached(disable_rsm_normals),
        }
    }

    *rsm_debug_cache = *rsm_debug;
}

fn enable_rsm_edges(
    mut commands: Commands,
    models: Query<Entity, With<crate::Model>>,
    children: Query<&Children>,
    model_primitives: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    mut gizmos: ResMut<Assets<GizmoAsset>>,
) {
    let mut cache: HashMap<bevy_asset::AssetId<Mesh>, bevy_asset::Handle<GizmoAsset>> =
        HashMap::new();
    for model in models {
        for model_primitive in children.iter_descendants(model) {
            let Ok(mesh_handle) = model_primitives.get(model_primitive) else {
                continue;
            };
            let gizmo = if let Some(cached) = cache.get(&mesh_handle.id()) {
                cached.clone()
            } else {
                let Some(mesh) = meshes.get(mesh_handle.id()) else {
                    unreachable!("Mesh handle must be valid.");
                };
                let Some(VertexAttributeValues::Float32x3(vertex)) =
                    mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                else {
                    continue;
                };

                let mut gizmo = GizmoAsset::new();

                if let Some(Indices::U16(indices)) = mesh.indices() {
                    for i in indices.chunks(3) {
                        gizmo.line(
                            Vec3::from_array(vertex[usize::from(i[0])]),
                            Vec3::from_array(vertex[usize::from(i[1])]),
                            palettes::css::ORANGE,
                        );
                        gizmo.line(
                            Vec3::from_array(vertex[usize::from(i[0])]),
                            Vec3::from_array(vertex[usize::from(i[2])]),
                            palettes::css::ORANGE,
                        );
                        gizmo.line(
                            Vec3::from_array(vertex[usize::from(i[1])]),
                            Vec3::from_array(vertex[usize::from(i[2])]),
                            palettes::css::ORANGE,
                        );
                    }
                } else {
                    for v in vertex.chunks(3) {
                        gizmo.line(
                            Vec3::from_array(v[0]),
                            Vec3::from_array(v[1]),
                            palettes::css::ORANGE,
                        );
                        gizmo.line(
                            Vec3::from_array(v[0]),
                            Vec3::from_array(v[2]),
                            palettes::css::ORANGE,
                        );
                        gizmo.line(
                            Vec3::from_array(v[1]),
                            Vec3::from_array(v[2]),
                            palettes::css::ORANGE,
                        );
                    }
                }

                let gizmo_handle = gizmos.add(gizmo);
                cache.insert(mesh_handle.id(), gizmo_handle.clone());
                gizmo_handle
            };

            commands.spawn((
                Transform::default(),
                Gizmo {
                    handle: gizmo,
                    ..Default::default()
                },
                ChildOf(model_primitive),
                EdgeGizmo(model_primitive),
            ));
        }
    }
}

fn disable_rsm_edges(mut commands: Commands, edge_gizmos: Query<Entity, With<EdgeGizmo>>) {
    for gizmo in edge_gizmos {
        commands.entity(gizmo).despawn();
    }
}

fn enable_rsm_normals(
    mut commands: Commands,
    models: Query<Entity, With<crate::Model>>,
    children: Query<&Children>,
    model_primitives: Query<&Mesh3d>,
    meshes: Res<Assets<Mesh>>,
    mut gizmos: ResMut<Assets<GizmoAsset>>,
) {
    let mut cache: HashMap<bevy_asset::AssetId<Mesh>, bevy_asset::Handle<GizmoAsset>> =
        HashMap::new();
    for model in models {
        for model_primitive in children.iter_descendants(model) {
            let Ok(mesh_handle) = model_primitives.get(model_primitive) else {
                continue;
            };
            let gizmo = if let Some(cached) = cache.get(&mesh_handle.id()) {
                cached.clone()
            } else {
                let Some(mesh) = meshes.get(mesh_handle.id()) else {
                    unreachable!("Mesh handle must be valid.");
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

                let mut gizmo = GizmoAsset::new();

                if let Some(Indices::U16(indices)) = mesh.indices() {
                    for i in indices {
                        let v = Vec3::from_array(vertex[usize::from(*i)]);
                        let n = Vec3::from_array(normals[usize::from(*i)]);
                        let color = Color::srgb_from_array(n.abs().to_array());
                        gizmo.line(v, v + n * NORMAL_GIZMOS_LENGTH, color);
                    }
                } else {
                    for (v, n) in vertex.iter().zip(normals) {
                        let v = Vec3::from_array(*v);
                        let n = Vec3::from_array(*n);
                        let color = Color::srgb_from_array(n.to_array());
                        gizmo.line(v, v + n * NORMAL_GIZMOS_LENGTH, color);
                    }
                }

                let gizmo_handle = gizmos.add(gizmo);
                cache.insert(mesh_handle.id(), gizmo_handle.clone());
                gizmo_handle
            };

            commands.spawn((
                Transform::default(),
                Gizmo {
                    handle: gizmo,
                    ..Default::default()
                },
                ChildOf(model_primitive),
                NormalGizmo(model_primitive),
            ));
        }
    }
}

fn disable_rsm_normals(mut commands: Commands, edge_gizmos: Query<Entity, With<NormalGizmo>>) {
    for gizmo in edge_gizmos {
        commands.entity(gizmo).despawn();
    }
}
