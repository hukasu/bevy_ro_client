use bevy::prelude::{Entity, Event};

use super::ActorFacing;

#[derive(Debug, Event)]
pub struct LoadActor {
    pub actor: String,
    pub facing: Option<ActorFacing>,
}

#[derive(Debug, Event)]
pub struct StartActor;

#[derive(Debug, Event)]
pub struct ActorTimerTick {
    pub entity: Entity,
}
