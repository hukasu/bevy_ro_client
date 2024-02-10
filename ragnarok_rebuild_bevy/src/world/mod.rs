mod components;
mod systems;

use bevy::app::{App, Plugin as BevyPlugin, Update};

pub use self::components::{Sounds, World};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, systems::clear_loaded_asset)
            .add_systems(Update, systems::set_ambient_light)
            .add_systems(Update, systems::spawn_directional_light)
            .add_systems(Update, systems::place_sounds);
    }
}
