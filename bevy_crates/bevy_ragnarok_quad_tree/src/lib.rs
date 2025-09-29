//! Quadtrees are a form of tree that have 4 children, they are used
//! to partition space for quicker look up of entities in a specific
//! region in the world.

pub mod plugin;

use bevy_ecs::{
    component::Component,
    entity::{Entity, EntityHashSet},
    reflect::ReflectComponent,
};
use bevy_reflect::Reflect;

/// A [`QuadTree`] has 4 children of type [`QuadTreeNode`], partitioning the space
/// in equal amounts on XZ.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[relationship_target(relationship = QuadTreeNode)]
pub struct QuadTree {
    nodes: Vec<Entity>,
}

/// A [`QuadTreeNode`] is a child of a [`QuadTree`], occupaying a quadrant of it.
///
/// A [`QuadTreeNode`] can be a [`QuadTree`] subdividing even further the space.
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[relationship(relationship_target = QuadTree)]
pub struct QuadTreeNode {
    parent: Entity,
}

/// Marks this entity to be tracked by the [`QuadTree`]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct TrackEntity;

/// Entities that are present on this [`QuadTreeNode`].
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[relationship_target(relationship = TrackedEntity)]
pub struct TrackingEntities {
    entities: EntityHashSet,
}

/// An entity that is located in a [`QuadTreeNode`]
#[cfg_attr(feature = "debug", derive(Debug))]
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[relationship(relationship_target = TrackingEntities)]
pub struct TrackedEntity {
    /// [`QuadTreeNode`] that this entity is on
    node: Entity,
}
