mod components;
// TODO remove pub after organizing the debug systems
mod audio;
mod camera;
pub mod entities;
mod loading_screen;
pub mod resources;
pub mod states;

use std::path::PathBuf;

use bevy::{
    app::{Startup, Update},
    asset::AssetServer,
    color::Color,
    ecs::children,
    math::{Quat, Vec3},
    prelude::{
        in_state, resource_changed, ClearColor, Commands, Entity, IntoScheduleConfigs, Name,
        NextState, Observer, OnAdd, Query, Res, ResMut, SpawnRelated, Transform, Trigger,
        Visibility, With,
    },
};

use ragnarok_rebuild_bevy::{
    assets::{gnd, paths},
    audio::{Bgm, Sound},
    WorldTransform,
};
use ragnarok_rsw::components::World;
use resources::LoadingWorld;
use states::GameState;

use self::components::Game;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ClearColor(Color::BLACK))
            // Plugins
            .add_plugins(audio::Plugin)
            .add_plugins(camera::Plugin)
            .add_plugins(entities::Plugin)
            .add_plugins(loading_screen::Plugin)
            .add_plugins(states::Plugin)
            // Startup system
            .add_systems(Startup, start_up)
            .add_systems(Update, skip_login.run_if(in_state(GameState::Login)))
            .add_systems(
                Update,
                update_world_transform.run_if(resource_changed::<gnd::GroundScale>),
            )
            // Observers
            .add_observer(attach_world_to_game)
            // TODO Change to observe on the the container entity
            // in 0.15
            .add_observer(attach_bgm_to_game)
            .add_observer(attach_sound_to_game);
    }
}

fn start_up(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.spawn((
        Name::new("RagnarokOnline"),
        components::Game,
        Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
        Visibility::default(),
        children![
            (
                Name::new("Playing sounds"),
                Transform::default(),
                Visibility::default(),
            ),
            (
                Name::new("Actors"),
                Transform::default(),
                Visibility::default(),
                Observer::new(attach_entity_to_game),
            )
        ],
    ));

    next_state.set(GameState::Login);
}

fn skip_login(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands.insert_resource(LoadingWorld {
        world: asset_server.load(PathBuf::from(paths::WORLD_FILES_FOLDER).join("prontera.rsw")),
    });

    next_state.set(GameState::MapChange);
}

fn attach_world_to_game(
    trigger: Trigger<OnAdd, World>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games.single().inspect_err(|err| bevy::log::error!("{err}")) else {
        return;
    };

    commands.entity(game).add_child(trigger.entity);
}

fn attach_entity_to_game(trigger: Trigger<OnAdd, entities::Entity>, mut commands: Commands) {
    commands
        .entity(trigger.observer())
        .add_child(trigger.entity);
}

fn attach_bgm_to_game(
    trigger: Trigger<OnAdd, Bgm>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games.single().inspect_err(|err| bevy::log::error!("{err}")) else {
        return;
    };

    commands.entity(game).add_child(trigger.entity);
}

fn attach_sound_to_game(
    trigger: Trigger<OnAdd, Sound>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games.single().inspect_err(|err| bevy::log::error!("{err}")) else {
        return;
    };

    commands.entity(game).add_child(trigger.entity);
}

fn update_world_transform(
    mut games: Query<&mut Transform, With<Game>>,
    ground_scale: Res<gnd::GroundScale>,
    mut world_transform: ResMut<WorldTransform>,
) {
    bevy::log::trace!("Updating world transform.");
    let Ok(mut game_transform) = games
        .single_mut()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    // TODO use ground scale
    *game_transform = game_transform.with_scale(Vec3::splat(**ground_scale));
    **world_transform = *game_transform;
}
