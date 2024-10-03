mod components;
// TODO remove pub after organizing the debug systems
pub mod entities;

use bevy::{
    app::{Plugin, Startup},
    core::Name,
    math::{Quat, Vec3},
    prelude::{
        BuildChildren, Commands, Entity, OnAdd, Query, ResMut, SpatialBundle, Transform, Trigger,
        With,
    },
};

use ragnarok_rebuild_bevy::{
    assets::{gnd::Ground, rsw::World},
    audio::{Bgm, Sound},
    WorldTransform,
};

use self::components::Game;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Resources
            .init_resource::<WorldTransform>()
            // Plugins
            .add_plugins(entities::Plugin)
            // Startup system
            .add_systems(Startup, start_up)
            // Observers
            .observe(attach_world_to_game)
            .observe(attach_entity_to_game)
            // TODO Change to observe on the the container entity
            // in 0.15
            .observe(attach_bgm_to_game)
            .observe(attach_sound_to_game)
            .observe(update_world_transform);
    }
}

fn start_up(mut commands: Commands) {
    commands
        .spawn((
            Name::new("RagnarokOnline"),
            components::Game,
            SpatialBundle {
                transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI))
                    .with_scale(Vec3::splat(0.2)),
                ..Default::default()
            },
        ))
        .with_children(|builder| {
            builder.spawn((Name::new("Playing sounds"), SpatialBundle::default()));
            builder.spawn((Name::new("Actors"), SpatialBundle::default()));
        });
}

fn attach_world_to_game(
    trigger: Trigger<OnAdd, World>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games
        .get_single()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    commands.entity(game).add_child(trigger.entity());
}

fn attach_entity_to_game(
    trigger: Trigger<OnAdd, entities::Entity>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games
        .get_single()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    commands.entity(game).add_child(trigger.entity());
}

fn attach_bgm_to_game(
    trigger: Trigger<OnAdd, Bgm>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games
        .get_single()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    commands.entity(game).add_child(trigger.entity());
}

fn attach_sound_to_game(
    trigger: Trigger<OnAdd, Sound>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games
        .get_single()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    commands.entity(game).add_child(trigger.entity());
}

fn update_world_transform(
    _trigger: Trigger<OnAdd, Ground>,
    mut games: Query<&mut Transform, With<Game>>,
    mut world_transform: ResMut<WorldTransform>,
) {
    bevy::log::trace!("Updating world transform.");
    let Ok(mut game_transform) = games
        .get_single_mut()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    // TODO use ground scale
    *game_transform = Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI))
        .with_scale(Vec3::splat(0.2));
    **world_transform = *game_transform;
}
