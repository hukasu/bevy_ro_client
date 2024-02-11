mod components;
mod systems;

use bevy::{
    app::{App, Plugin as BevyPlugin, Update},
    ecs::schedule::IntoSystemConfigs,
};

pub use self::components::{Sounds, World};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            // Events
            .add_event::<systems::RSWCompletedLoading>()
            // Systems
            .add_systems(Update, systems::filter_events_that_are_tied_to_a_map)
            .add_systems(
                Update,
                systems::clear_loaded_asset.after(systems::filter_events_that_are_tied_to_a_map),
            )
            .add_systems(
                Update,
                systems::set_ambient_light.after(systems::filter_events_that_are_tied_to_a_map),
            )
            .add_systems(
                Update,
                systems::spawn_directional_light
                    .after(systems::filter_events_that_are_tied_to_a_map),
            )
            .add_systems(
                Update,
                systems::place_sounds.after(systems::filter_events_that_are_tied_to_a_map),
            );
    }
}
