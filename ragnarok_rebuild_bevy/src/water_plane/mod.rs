mod component;
mod system;

use bevy::app::{App, Plugin as BevyPlugin, Update};

pub use component::WaterPlane;

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            // Systems
            .add_systems(Update, system::update_texture);
    }
}
