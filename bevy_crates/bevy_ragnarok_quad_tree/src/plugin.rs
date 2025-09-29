//! Sets up the [`QuadTree`].

use bevy_app::PostUpdate;
use bevy_camera::primitives::Aabb;
use bevy_ecs::{
    entity::Entity,
    lifecycle::Add,
    observer::On,
    query::{With, Without},
    relationship::{Relationship, RelationshipTarget},
    schedule::IntoScheduleConfigs,
    system::{Commands, Query, Single},
};
use bevy_log::{error, trace};
use bevy_math::Vec3A;
use bevy_transform::{TransformSystems, components::GlobalTransform};

use crate::{QuadTree, QuadTreeNode, TrackEntity, TrackedEntity};

/// Add systems to update [`QuadTree`].
#[cfg_attr(
    feature = "reflect",
    doc = "Also register reflection of quadtree types."
)]
pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            PostUpdate,
            update_tracked_entities.after(TransformSystems::Propagate),
        );
        app.add_observer(initial_tracking);

        app.register_type::<TrackEntity>();
        #[cfg(feature = "reflect")]
        {
            use crate::TrackingEntities;

            app.register_type::<QuadTree>();
            app.register_type::<QuadTreeNode>();
            app.register_type::<TrackedEntity>();
            app.register_type::<TrackingEntities>();
        }
    }
}

/// Updates the [`QuadTreeNode`] that the [`TrackEntity`]
/// points to.
fn update_tracked_entities(
    mut commands: Commands,
    entities: Query<(Entity, &GlobalTransform, Option<&TrackedEntity>), With<TrackEntity>>,
    root: Single<(Entity, &Aabb, &GlobalTransform, &QuadTree), Without<QuadTreeNode>>,
    nodes: Query<(Entity, &Aabb, &GlobalTransform, Option<&QuadTree>), With<QuadTreeNode>>,
) {
    trace!("Updating tracked entities.");
    for (entity, transform, tracked_entity_opt) in entities {
        do_tracking(
            &mut commands,
            entity,
            transform,
            tracked_entity_opt,
            &root,
            nodes,
        );
    }
}

/// Adds an entity to the [`QuadTree`] on insertion of [`TrackEntity`].
fn initial_tracking(
    event: On<Add, TrackEntity>,
    mut commands: Commands,
    entities: Query<(&GlobalTransform, Option<&TrackedEntity>), With<TrackEntity>>,
    root: Single<(Entity, &Aabb, &GlobalTransform, &QuadTree), Without<QuadTreeNode>>,
    nodes: Query<(Entity, &Aabb, &GlobalTransform, Option<&QuadTree>), With<QuadTreeNode>>,
) {
    let entity = event.entity;
    let Ok((transform, tracked_entity_opt)) = entities.get(entity) else {
        unreachable!("Couldn't add {entity} to Quadtree due to missing components.",);
    };

    do_tracking(
        &mut commands,
        entity,
        transform,
        tracked_entity_opt,
        &root,
        nodes,
    );
}

/// Places an entity into a [`QuadTreeNode`]
fn do_tracking(
    commands: &mut Commands,
    entity: Entity,
    transform: &GlobalTransform,
    tracked_entity_opt: Option<&TrackedEntity>,
    root: &(Entity, &Aabb, &GlobalTransform, &QuadTree),
    nodes: Query<(Entity, &Aabb, &GlobalTransform, Option<&QuadTree>), With<QuadTreeNode>>,
) {
    let translation = transform.translation();
    if tracked_entity_opt
        .and_then(|tracked_entity| nodes.get(tracked_entity.get()).ok())
        .filter(|(_, aabb, node_transform, quad_tree_opt)| {
            if quad_tree_opt.is_some() {
                unreachable!("All entities must be tracked by a leaf node.");
            } else {
                let transformed_aabb = TransformedAabb::new(aabb, node_transform);
                transformed_aabb.contains(translation)
            }
        })
        .is_none()
    {
        trace!("{entity} needs updating its location in the Quadtree.");
        commands.entity(entity).remove::<TrackedEntity>();

        let root_transformed_aabb = TransformedAabb::new(root.1, root.2);
        if !root_transformed_aabb.contains(translation) {
            error!("{entity} is off the QuadTree.");
            return;
        }

        let mut current_node = root.0;
        let mut child_nodes = root.3.collection();
        loop {
            let Some((child_node, _, _, child_node_children_opt)) = child_nodes
                .iter()
                .filter_map(|child_node| nodes.get(*child_node).ok())
                .find(|(_, aabb, node_transform, _)| {
                    let transformed_aabb = TransformedAabb::new(aabb, node_transform);
                    transformed_aabb.contains(translation)
                })
            else {
                error!("{entity} is not in any of the nodes of {current_node}.");
                break;
            };

            current_node = child_node;
            if let Some(child_node_children) = child_node_children_opt {
                child_nodes = child_node_children.collection();
            } else {
                commands
                    .entity(entity)
                    .insert(<TrackedEntity as Relationship>::from(current_node));
                break;
            }
        }
    }
}

/// An [`Aabb`] that had its center and half extents transformed
/// by a [`GlobalTransform`].
struct TransformedAabb(Aabb);

impl TransformedAabb {
    /// Create an instance of [`TransformedAabb`] from a [`Aabb`] and [`GlobalTransform`].
    fn new(aabb: &Aabb, global_transform: &GlobalTransform) -> Self {
        Self(Aabb {
            center: global_transform.transform_point(aabb.center.into()).into(),
            half_extents: aabb.half_extents * global_transform.scale().to_vec3a(),
        })
    }

    /// Checks if a point is inside this [`TransformedAabb`]
    fn contains(&self, point: impl Into<Vec3A>) -> bool {
        let point = point.into();
        let aabb = self.0;

        ((aabb.center.x - point.x).abs() <= aabb.half_extents.x)
            && ((aabb.center.y - point.y).abs() <= aabb.half_extents.y)
            && ((aabb.center.z - point.z).abs() <= aabb.half_extents.z)
    }
}
