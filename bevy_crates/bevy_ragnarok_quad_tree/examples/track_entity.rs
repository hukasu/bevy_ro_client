//! Shows a [`Quadtree`] tracking an entity
#![expect(clippy::unwrap_used, reason = "This is an example")]

use std::f32::consts::PI;

use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::AssetServer,
    camera::{Camera3d, primitives::Aabb},
    color::{Color, palettes},
    ecs::{
        entity::Entity,
        lifecycle::{Insert, Replace},
        observer::On,
        query::With,
        relationship::Relationship,
        system::{Commands, Query, Res, Single},
    },
    gizmos::{GizmoAsset, aabb::ShowAabbGizmo, retained::Gizmo},
    light::DirectionalLight,
    math::{Isometry3d, Quat, Vec2, Vec3, Vec3A, primitives::Plane3d},
    mesh::{Mesh3d, MeshBuilder, Meshable},
    pbr::{MeshMaterial3d, StandardMaterial},
    time::Time,
    transform::components::Transform,
};
use bevy_ragnarok_quad_tree::{QuadTreeNode, TrackEntity, TrackedEntity};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(bevy_ragnarok_quad_tree::plugin::Plugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, bob);
    app.add_observer(update_gizmo_color_on_insert);
    app.add_observer(update_gizmo_color_on_replace);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 10., 2.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(0., 10., 10.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(asset_server.add(Plane3d::new(Vec3::Y, Vec2::splat(3.)).mesh().build())),
        MeshMaterial3d(asset_server.add(StandardMaterial::from_color(Color::WHITE))),
    ));

    let mut gizmo = GizmoAsset::new();
    gizmo.sphere(
        Isometry3d::from_xyz(0., 0., 0.),
        0.25,
        palettes::tailwind::RED_300,
    );

    commands.spawn((
        TrackEntity,
        Transform::default(),
        Gizmo {
            handle: asset_server.add(gizmo),
            ..Default::default()
        },
    ));

    let world = commands
        .spawn(Transform::from_rotation(
            // This is to simulate the fact that Ragnarok Online uses Y down
            Quat::from_rotation_x(PI),
        ))
        .id();

    let root = setup_quadtree(
        &mut commands,
        Vec3::default(),
        Aabb {
            center: Vec3A::new(0., -2.5, 0.),
            half_extents: Vec3A::new(3., 2.5, 3.),
        },
        None,
        &[
            palettes::tailwind::CYAN_200.into(),
            palettes::tailwind::CYAN_400.into(),
            palettes::tailwind::CYAN_600.into(),
        ],
    )
    .unwrap();

    commands.entity(world).add_child(root);
}

fn bob(mut entity: Single<&mut Transform, With<TrackEntity>>, time: Res<Time>) {
    entity.translation = Vec3::new(
        (time.elapsed_secs() / 2.).sin() * 3.,
        1.5,
        (time.elapsed_secs() / 5.).sin() * 3.,
    );
}

fn update_gizmo_color_on_insert(
    event: On<Insert, TrackedEntity>,
    entities: Query<&TrackedEntity, With<TrackEntity>>,
    mut nodes: Query<&mut ShowAabbGizmo>,
) {
    let tracked_entity = entities.get(event.entity).unwrap();
    let mut gizmo = nodes.get_mut(tracked_entity.get()).unwrap();
    gizmo.color = Some(palettes::tailwind::RED_600.into());
}

fn update_gizmo_color_on_replace(
    event: On<Replace, TrackedEntity>,
    entities: Query<&TrackedEntity, With<TrackEntity>>,
    mut nodes: Query<&mut ShowAabbGizmo>,
) {
    let tracked_entity = entities.get(event.entity).unwrap();
    let mut gizmo = nodes.get_mut(tracked_entity.get()).unwrap();
    gizmo.color = Some(palettes::tailwind::CYAN_600.into());
}

fn setup_quadtree(
    commands: &mut Commands,
    translation: Vec3,
    aabb: Aabb,
    parent_opt: Option<Entity>,
    colors: &[Color],
) -> Option<Entity> {
    if colors.is_empty() {
        return None;
    }

    let current_node = commands
        .spawn((
            Transform::from_translation(translation),
            aabb,
            ShowAabbGizmo {
                color: Some(colors[0]),
            },
        ))
        .id();
    if let Some(parent) = parent_opt {
        commands
            .entity(current_node)
            .insert(<QuadTreeNode as Relationship>::from(parent));
    }

    let children = [
        setup_quadtree(
            commands,
            Vec3::new(aabb.half_extents.x / 2., 0., aabb.half_extents.z / 2.),
            Aabb {
                center: aabb.center * Vec3A::new(1., 0.4, 1.),
                half_extents: aabb.half_extents * Vec3A::new(0.5, 0.4, 0.5),
            },
            Some(current_node),
            &colors[1..],
        ),
        setup_quadtree(
            commands,
            Vec3::new(aabb.half_extents.x / 2., 0., aabb.half_extents.z / -2.),
            Aabb {
                center: aabb.center * Vec3A::new(1., 0.5, 1.),
                half_extents: aabb.half_extents * Vec3A::new(0.5, 0.5, 0.5),
            },
            Some(current_node),
            &colors[1..],
        ),
        setup_quadtree(
            commands,
            Vec3::new(aabb.half_extents.x / -2., 0., aabb.half_extents.z / 2.),
            Aabb {
                center: aabb.center * Vec3A::new(1., 0.75, 1.),
                half_extents: aabb.half_extents * Vec3A::new(0.5, 0.75, 0.5),
            },
            Some(current_node),
            &colors[1..],
        ),
        setup_quadtree(
            commands,
            Vec3::new(aabb.half_extents.x / -2., 0., aabb.half_extents.z / -2.),
            Aabb {
                center: aabb.center,
                half_extents: aabb.half_extents * Vec3A::new(0.5, 1., 0.5),
            },
            Some(current_node),
            &colors[1..],
        ),
    ];
    commands
        .entity(current_node)
        .add_children(&children.into_iter().flatten().collect::<Vec<_>>());

    Some(current_node)
}
