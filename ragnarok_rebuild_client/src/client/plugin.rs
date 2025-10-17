use bevy::{
    app::{Startup, Update},
    color::Color,
    ecs::{children, lifecycle::Add, observer::On},
    math::Vec3,
    prelude::{
        in_state, resource_changed, ClearColor, Commands, Entity, IntoScheduleConfigs, Name,
        NextState, Observer, Query, Res, ResMut, SpawnRelated, Transform, Visibility, With,
    },
};

use bevy_ragnarok_gnd::GroundScale;
use ragnarok_rebuild_bevy::audio::{Bgm, Sound};

use crate::client::world::ChangeMap;

use super::{audio, camera, entities, loading_screen, states, states::GameState, world, Game};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ClearColor(Color::BLACK))
            // Plugins
            .add_plugins(audio::Plugin)
            .add_plugins(camera::plugin::Plugin)
            .add_plugins(entities::Plugin)
            .add_plugins(loading_screen::Plugin)
            .add_plugins(states::Plugin)
            .add_plugins(world::plugin::Plugin)
            // Startup system
            .add_systems(Startup, start_up)
            .add_systems(Update, skip_login.run_if(in_state(GameState::Login)))
            .add_systems(
                Update,
                update_world_transform.run_if(resource_changed::<GroundScale>),
            )
            // Observers
            // TODO Change to observe on the the container entity
            // in 0.15
            .add_observer(attach_bgm_to_game)
            .add_observer(attach_sound_to_game);
    }
}

fn start_up(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
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

    next_state.set(GameState::Login);
}

fn skip_login(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    commands.trigger(ChangeMap::new("prontera.rsw"));
    next_state.set(GameState::MapChange);
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

fn update_world_transform(
    mut games: Query<&mut Transform, With<Game>>,
    ground_scale: Res<GroundScale>,
) {
    bevy::log::trace!("Updating world transform.");
    let Ok(mut game_transform) = games
        .single_mut()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    // TODO use ground scale
    *game_transform =
        game_transform.with_scale(Vec3::splat(**ground_scale) * Vec3::new(1., -1., -1.));
}
