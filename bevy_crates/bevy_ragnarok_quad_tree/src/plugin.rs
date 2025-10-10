//! Sets up the [`QuadTree`].

use bevy_app::{AppExit, Update};
use bevy_camera::primitives::Aabb;
use bevy_ecs::{
    entity::Entity,
    name::NameOrEntity,
    query::{Changed, With, Without},
    relationship::{Relationship, RelationshipTarget},
    schedule::IntoScheduleConfigs,
    system::{Commands, Populated, Query, RegisteredSystemError, Single},
    world::World,
};
use bevy_gizmos::aabb::ShowAabbGizmo;
use bevy_log::{error, trace};
use bevy_math::bounding::{Aabb3d, BoundingVolume};
use bevy_transform::components::GlobalTransform;

use crate::{QuadTree, QuadTreeNode, TrackEntity, TrackedEntity, TrackingEntities};

/// Add systems to update [`QuadTree`].
#[cfg_attr(
    feature = "reflect",
    doc = "Also register reflection of quadtree types."
)]
pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            Update,
            ((initial_insertion, reinsert_on_root), execute_subsystems).chain(),
        );

        app.register_type::<TrackEntity>();
        app.register_type::<QuadTree>();
        app.register_type::<QuadTreeNode>();
        #[cfg(feature = "reflect")]
        {
            use crate::TrackingEntities;

            app.register_type::<TrackedEntity>();
            app.register_type::<TrackingEntities>();
        }
    }
}

#[expect(clippy::type_complexity, reason = "Queries are complex")]
fn initial_insertion(
    mut commands: Commands,
    tracked_entities: Populated<
        (NameOrEntity, &GlobalTransform),
        (With<TrackEntity>, Without<TrackedEntity>),
    >,
    quad_tree: Single<
        (NameOrEntity, &GlobalTransform, &Aabb),
        (With<QuadTree>, Without<QuadTreeNode>),
    >,
) {
    trace!("Inserting {} on root.", tracked_entities.iter().len());
    let (quad_tree, quad_tree_global_transform, quad_tree_aabb) = quad_tree.into_inner();
    for (tracked_entity, global_transform) in tracked_entities.into_inner() {
        let quad_tree_aabb = Aabb3d::new(quad_tree_aabb.center, quad_tree_aabb.half_extents)
            .scale_around_center(quad_tree_global_transform.scale())
            .rotated_by(quad_tree_global_transform.rotation());
        let position = global_transform.translation();
        if quad_tree_aabb.closest_point(position) == position.to_vec3a() {
            commands
                .entity(tracked_entity.entity)
                .insert(<TrackedEntity as Relationship>::from(quad_tree.entity));
        } else {
            error!("{tracked_entity} is outside of {quad_tree}.");
        }
    }
}

#[expect(clippy::type_complexity, reason = "Queries are complex")]
fn reinsert_on_root(
    mut commands: Commands,
    tracked_entities: Populated<
        (NameOrEntity, &GlobalTransform),
        (
            With<TrackEntity>,
            With<TrackedEntity>,
            Changed<GlobalTransform>,
        ),
    >,
    quad_tree: Single<
        (NameOrEntity, &GlobalTransform, &Aabb),
        (With<QuadTree>, Without<QuadTreeNode>),
    >,
) {
    trace!("Reinserting {} on root.", tracked_entities.iter().count());
    let (quad_tree, quad_tree_global_transform, quad_tree_aabb) = quad_tree.into_inner();
    for (tracked_entity, global_transform) in tracked_entities.into_inner() {
        let quad_tree_aabb = Aabb3d::new(quad_tree_aabb.center, quad_tree_aabb.half_extents)
            .scale_around_center(quad_tree_global_transform.scale())
            .rotated_by(quad_tree_global_transform.rotation());
        let position = global_transform.translation();
        if quad_tree_aabb.closest_point(position) == position.to_vec3a() {
            commands
                .entity(tracked_entity.entity)
                .insert(<TrackedEntity as Relationship>::from(quad_tree.entity));
        } else {
            error!("{tracked_entity} is outside of {quad_tree}.");
        }
    }
}

fn execute_subsystems(world: &mut World) {
    trace!("Executing subsystems.");
    for _ in 0..5 {
        match world.run_system_cached(push_tracked_entities_to_child_nodes) {
            Ok(_) => (),
            Err(RegisteredSystemError::Skipped(_)) => break,
            Err(err) => {
                error!("{err}");
                world.write_message(AppExit::from_code(1));
                break;
            }
        }
    }
}

fn push_tracked_entities_to_child_nodes(
    mut commands: Commands,
    quad_tree_nodes: Populated<(NameOrEntity, &QuadTree, &TrackingEntities)>,
    tracked_entities: Query<(NameOrEntity, &GlobalTransform), With<TrackEntity>>,
    aabbs: Query<(&Aabb, &GlobalTransform), With<QuadTreeNode>>,
) {
    for (quad_tree, child_nodes, tracking_entities) in quad_tree_nodes.into_inner() {
        let Ok(children): Result<[Entity; 4], _> = child_nodes.collection().clone().try_into()
        else {
            unreachable!("QuadTree must always have 4 children.");
        };
        let Ok(child_nodes_aabbs) = aabbs.get_many(children) else {
            unreachable!("Relationship invariant was broken.");
        };
        let child_nodes_aabbs = child_nodes_aabbs.map(|(aabb, global_transform)| {
            Aabb3d::new(aabb.center, aabb.half_extents)
                .scale_around_center(global_transform.scale())
                .rotated_by(global_transform.rotation())
        });
        trace!(
            "{quad_tree} has {} needing to be pushed to leaf.",
            tracking_entities.collection().len()
        );

        'entity: for (tracked_entity, global_transform) in
            tracked_entities.iter_many(tracking_entities.collection())
        {
            for (aabb, quad_tree_child_node) in child_nodes_aabbs.iter().zip(children) {
                let position = global_transform.translation();
                if aabb.closest_point(position) == position.to_vec3a() {
                    commands
                        .entity(tracked_entity.entity)
                        .insert(<TrackedEntity as Relationship>::from(quad_tree_child_node))
                        .remove::<ShowAabbGizmo>();
                    continue 'entity;
                }
            }
            commands
                .entity(tracked_entity.entity)
                .insert(ShowAabbGizmo::default());
            // error!("{tracked_entity} is not in any of the children of {quad_tree}.");
        }
    }
}
