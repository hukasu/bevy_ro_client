use bevy::{
    app::{Startup, Update},
    color::Color,
    ecs::{children, lifecycle::Add, observer::On},
    prelude::{
        in_state, ClearColor, Commands, Entity, IntoScheduleConfigs, Name, Observer, Query,
        SpawnRelated, Transform, Visibility, With,
    },
    state::app::AppExtStates,
};

use ragnarok_rebuild_bevy::audio::{Bgm, Sound};

use super::{audio, camera, entities, loading_screen, world, Game};
use crate::client::{world::ChangeMap, GameStates};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_computed_state::<GameStates>();

        app.insert_resource(ClearColor(Color::BLACK))
            // Plugins
            .add_plugins(audio::Plugin)
            .add_plugins(camera::plugin::Plugin)
            .add_plugins(entities::Plugin)
            .add_plugins(loading_screen::Plugin)
            .add_plugins(world::plugin::Plugin)
            // Startup system
            .add_systems(Startup, start_up)
            .add_systems(Update, skip_login.run_if(in_state(GameStates::Login)))
            // Observers
            // TODO Change to observe on the the container entity
            // in 0.15
            .add_observer(attach_bgm_to_game)
            .add_observer(attach_sound_to_game);
    }
}

fn start_up(mut commands: Commands) {
    commands.spawn((
        Name::new("RagnarokOnline"),
        Game,
        Transform::default(),
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
}

fn skip_login(mut commands: Commands) {
    commands.trigger(ChangeMap::new("prontera.rsw"));
}

fn attach_entity_to_game(event: On<Add, entities::Entity>, mut commands: Commands) {
    commands.entity(event.observer()).add_child(event.entity);
}

fn attach_bgm_to_game(
    event: On<Add, Bgm>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games.single().inspect_err(|err| bevy::log::error!("{err}")) else {
        return;
    };

    commands.entity(game).add_child(event.entity);
}

fn attach_sound_to_game(
    event: On<Add, Sound>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games.single().inspect_err(|err| bevy::log::error!("{err}")) else {
        return;
    };

    commands.entity(game).add_child(event.entity);
}
