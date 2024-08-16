mod event;

use std::borrow::Cow;

use bevy::{
    app::Plugin,
    asset::{AssetServer, Handle},
    core::Name,
    ecs::query::QuerySingleError,
    math::{Quat, Vec3},
    prelude::{
        Commands, DespawnRecursiveExt, Entity, Query, Res, SpatialBundle, Transform, Trigger, With,
    },
    scene::Scene,
};

use ragnarok_rebuild_bevy::assets::rsw;

pub use self::event::ChangeWorld;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Observers
            .observe(change_world);
    }
}

fn change_world(
    trigger: Trigger<ChangeWorld>,
    mut commands: Commands,
    worlds: Query<Entity, With<rsw::World>>,
    asset_loader: Res<AssetServer>,
) {
    match worlds.get_single() {
        Ok(world) => {
            // Remove old world
            commands.entity(world).despawn_recursive();
        }
        Err(QuerySingleError::NoEntities(_)) => (),
        Err(QuerySingleError::MultipleEntities(err)) => {
            bevy::log::error!("There where multiple `rsw::World`s spawned. '{:?}'", err);
            return;
        }
    }

    // Spawn new world
    let next_world_handle: Handle<Scene> =
        asset_loader.load(trigger.event().next_world.clone().to_string());
    commands.spawn((
        Name::new(Cow::Owned(trigger.event().next_world.clone().into())),
        rsw::World,
        next_world_handle,
        SpatialBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI))
                .with_scale(Vec3::splat(0.2)),
            ..Default::default()
        },
    ));
}
