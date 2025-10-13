//! Relationships for quick access of children of
//! [`World`](crate::World).

use bevy_ecs::{component::Component, entity::Entity, reflect::ReflectComponent};
use bevy_reflect::Reflect;

/// Link between the animated props container and the [`World`](crate::World) from
/// which they were loaded from.
#[derive(Debug, Reflect, Component)]
#[reflect(Component)]
#[relationship(relationship_target=WorldOfModels)]
pub struct ModelsOfWorld(Entity);

/// [`World`](crate::World) that contains animated props.
#[derive(Debug, Reflect, Component)]
#[reflect(Component)]
#[relationship_target(relationship=ModelsOfWorld)]
pub struct WorldOfModels(Entity);
