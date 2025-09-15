use bevy::prelude::States;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Startup,
    Login,
    MapChange,
    Game,
}
