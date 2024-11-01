mod component;
mod material;

use bevy::{
    app::{App, Plugin as BevyPlugin, Update},
    prelude::{Commands, Entity, Query, Res},
    time::Time,
};

pub use self::{
    component::WaterPlane,
    material::{WaterPlaneMaterial, Wave},
};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app
            // Register Types
            .register_type::<WaterPlane>()
            // Systems
            .add_systems(Update, update_texture)
            // Material
            .add_plugins(material::Plugin);
    }
}

pub fn update_texture(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WaterPlane)>,
    timer: Res<Time>,
) {
    for (entity, mut plane) in query.iter_mut() {
        plane.timer.tick(timer.delta());
        if plane.timer.just_finished() {
            plane.current_frame += 1;
            plane.current_frame %= 32;

            commands
                .entity(entity)
                .insert(plane.frames[plane.current_frame].clone());
        }
    }
}
