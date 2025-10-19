use bevy_app::{App, Update};
use bevy_asset::{Assets, Handle, uuid_handle};
use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res},
};
use bevy_math::{Vec2, Vec3, primitives::Plane3d};
use bevy_mesh::Mesh;
use bevy_pbr::MeshMaterial3d;
use bevy_time::Time;

use crate::{WaterPlane, material};

const WATER_PLANE_MESH: Handle<Mesh> = uuid_handle!("7a77a34b-40ea-42ec-b935-1b57b38b17d7");

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

    fn finish(&self, app: &mut App) {
        if let Err(err) = app.world_mut().resource_mut::<Assets<Mesh>>().insert(
            &WATER_PLANE_MESH,
            Plane3d::new(Vec3::NEG_Y, Vec2::splat(0.5)).into(),
        ) {
            unreachable!("Should never error for Uuid handles. `{err}`");
        };
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
