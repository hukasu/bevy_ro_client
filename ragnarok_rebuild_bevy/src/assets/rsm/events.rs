use bevy::prelude::Event;

#[derive(Debug, Event)]
pub struct StartPropAnimation {
    pub speed: f32,
    pub mode: i32,
}
