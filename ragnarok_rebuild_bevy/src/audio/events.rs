use std::borrow::Cow;

use bevy::prelude::Event;

#[derive(Debug, Event)]
pub struct PlayBgm {
    pub track: Cow<'static, str>,
}
