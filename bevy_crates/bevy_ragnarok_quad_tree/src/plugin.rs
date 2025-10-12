//! Sets up the [`QuadTree`].

use bevy_app::{AppExit, Update};
use bevy_camera::primitives::Aabb;
use bevy_ecs::{
    name::NameOrEntity,
    query::{Changed, With, Without},
    relationship::{Relationship, RelationshipTarget},
    schedule::IntoScheduleConfigs,
    system::{Commands, Populated, Query, RegisteredSystemError, Single},
    world::World,
};
use bevy_gizmos::aabb::ShowAabbGizmo;
use bevy_log::{error, trace};
use bevy_math::Vec3A;
use bevy_transform::components::{GlobalTransform, Transform};

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
        (NameOrEntity, &Transform),
        (With<TrackEntity>, Without<TrackedEntity>),
    >,
    quad_tree: Single<(NameOrEntity, &Aabb), (With<QuadTree>, Without<QuadTreeNode>)>,
) {
    trace!("Inserting {} on root.", tracked_entities.iter().len());
    let (quad_tree, quad_tree_aabb) = quad_tree.into_inner();
    for (tracked_entity, transform) in tracked_entities.into_inner() {
        if quad_tree_aabb.contains_point(transform.translation) {
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
        (NameOrEntity, &Transform),
        (
            With<TrackEntity>,
            With<TrackedEntity>,
            Changed<GlobalTransform>,
        ),
    >,
    quad_tree: Single<(NameOrEntity, &Aabb), (With<QuadTree>, Without<QuadTreeNode>)>,
) {
    trace!("Reinserting {} on root.", tracked_entities.iter().count());
    let (quad_tree, quad_tree_aabb) = quad_tree.into_inner();
    for (tracked_entity, transform) in tracked_entities.into_inner() {
        if quad_tree_aabb.contains_point(transform.translation) {
            commands
                .entity(tracked_entity.entity)
                .insert(<TrackedEntity as Relationship>::from(quad_tree.entity));
        } else {
            error!("{tracked_entity} is outside of {quad_tree}.");
        }
    }
}

fn execute_subsystems(world: &mut World) {
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
    quad_trees: Populated<(NameOrEntity, &QuadTree, &TrackingEntities)>,
    quad_tree_nodes: Query<(NameOrEntity, &Aabb), With<QuadTreeNode>>,
    tracked_entities: Query<(NameOrEntity, &Transform), With<TrackEntity>>,
) {
    for (quad_tree, child_nodes, tracking_entities) in quad_trees.into_inner() {
        trace!(
            "{quad_tree} has {} needing to be pushed towards leaf.",
            tracking_entities.len()
        );

        'entity: for (tracked_entity, transform) in
            tracked_entities.iter_many(tracking_entities.collection())
        {
            let point = transform.translation;
            for (quad_tree_node, quad_tree_node_aabb) in
                quad_tree_nodes.iter_many(child_nodes.collection())
            {
                if quad_tree_node_aabb.contains_point(point) {
                    commands
                        .entity(tracked_entity.entity)
                        .insert(<TrackedEntity as Relationship>::from(quad_tree_node.entity))
                        .remove::<ShowAabbGizmo>();
                    continue 'entity;
                }
            }
            commands
                .entity(tracked_entity.entity)
                .insert(ShowAabbGizmo::default());
        }
    }
}

trait ContainsPointExt {
    fn contains_point(&self, point: impl Into<Vec3A>) -> bool;
}

impl ContainsPointExt for Aabb {
    fn contains_point(&self, point: impl Into<Vec3A>) -> bool {
        const EPSILON: f32 = 0.00001;
        let point = point.into();

        let min = self.min();
        let max = self.max();

        let x_test = (min.x - EPSILON) <= point.x && point.x <= (max.x + EPSILON);
        // There are way too many Gat tiles that do not fit in a node.
        // Treating each node as being 2d on XZ.
        // let y_test = (min.y - EPSILON) <= point.y && point.y <= (max.y + EPSILON);
        let z_test = (min.z - EPSILON) <= point.z && point.z <= (max.z + EPSILON);

        x_test && z_test
    }
}
