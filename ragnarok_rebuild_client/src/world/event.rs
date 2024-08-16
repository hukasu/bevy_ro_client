use bevy::prelude::Event;

#[derive(Debug, Event)]
pub struct ChangeWorld {
    pub next_world: Box<str>,
}
