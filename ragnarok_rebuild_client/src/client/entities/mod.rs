mod components;

pub use self::components::Entity;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<Entity>();
    }
}
