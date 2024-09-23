use bevy::prelude::SystemSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash, SystemSet)]
pub enum TableSystems {
    TableStartup,
}
