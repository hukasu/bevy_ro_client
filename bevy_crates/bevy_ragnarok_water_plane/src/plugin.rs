use bevy_app::{App, Update};
use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res},
};
use bevy_pbr::MeshMaterial3d;
use bevy_time::Time;

use crate::{WaterPlane, material};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // Systems
        app.add_systems(Update, update_texture);

        // Material
        app.add_plugins(material::plugin::Plugin);

        // Register Types
        app.register_type::<WaterPlane>();
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
                .try_insert(MeshMaterial3d(plane.frames[plane.current_frame].clone()));
        }
    }
}
