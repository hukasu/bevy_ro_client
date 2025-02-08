mod components;
// TODO remove pub after organizing the debug systems
mod camera;
pub mod entities;

use bevy::{
    app::{Plugin, Startup, Update},
    core::Name,
    math::{Quat, Vec3},
    prelude::{
        resource_changed, BuildChildren, ChildBuild, Commands, Entity, IntoSystemConfigs, OnAdd,
        Query, Res, ResMut, Transform, Trigger, Visibility, With,
    },
};

use ragnarok_rebuild_bevy::{
    assets::{gnd, rsw},
    audio::{Bgm, Sound},
    WorldTransform,
};

use self::components::Game;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Plugins
            .add_plugins(entities::Plugin)
            .add_plugins(camera::Plugin)
            // Startup system
            .add_systems(Startup, start_up)
            .add_systems(
                Update,
                update_world_transform.run_if(resource_changed::<gnd::GroundScale>),
            )
            // Observers
            .add_observer(attach_world_to_game)
            .add_observer(attach_entity_to_game)
            // TODO Change to observe on the the container entity
            // in 0.15
            .add_observer(attach_bgm_to_game)
            .add_observer(attach_sound_to_game);
    }
}

fn start_up(mut commands: Commands) {
    commands
        .spawn((
            Name::new("RagnarokOnline"),
            components::Game,
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
            Visibility::default(),
        ))
        .with_children(|builder| {
            builder.spawn((
                Name::new("Playing sounds"),
                Transform::default(),
                Visibility::default(),
            ));
            builder.spawn((
                Name::new("Actors"),
                Transform::default(),
                Visibility::default(),
            ));
        });
}

fn attach_world_to_game(
    trigger: Trigger<OnAdd, rsw::World>,
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
    mut games: Query<&mut Transform, With<Game>>,
    ground_scale: Res<gnd::GroundScale>,
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
    *game_transform = game_transform.with_scale(Vec3::splat(**ground_scale));
    **world_transform = *game_transform;
}
