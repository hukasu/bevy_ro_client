use bevy::{
    app::{AppExit, Startup, Update},
    asset::{AssetServer, Assets, RecursiveDependencyLoadState},
    color::Color,
    ecs::{
        children, lifecycle::Add, observer::On, schedule::common_conditions::resource_exists,
        system::Single,
    },
    log::error,
    math::Vec3,
    prelude::{
        in_state, resource_changed, ClearColor, Commands, Entity, IntoScheduleConfigs, Name,
        NextState, Observer, Query, Res, ResMut, SpawnRelated, Transform, Visibility, With,
    },
    scene::SceneSpawner,
};

use bevy_ragnarok_rsw::{assets::RswWorld, World};
use ragnarok_rebuild_bevy::{
    assets::{gnd, paths},
    audio::{Bgm, Sound},
};

use super::{
    audio, camera, entities, loading_screen, resources::LoadingWorld, states, states::GameState,
    Game,
};

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

        app.add_systems(Update, swap_world.run_if(resource_exists::<LoadingWorld>));
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

fn skip_login(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands.insert_resource(LoadingWorld {
        world: asset_server.load(format!("{}prontera.rsw", paths::WORLD_FILES_FOLDER)),
    });

    next_state.set(GameState::MapChange);
}

fn attach_world_to_game(
    event: On<Add, World>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games.single().inspect_err(|err| bevy::log::error!("{err}")) else {
        return;
    };

    commands.entity(game).add_child(event.entity);
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
    ground_scale: Res<gnd::GroundScale>,
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

fn swap_world(
    mut commands: Commands,
    previous_world: Option<Single<Entity, With<World>>>,
    loading_world: Res<LoadingWorld>,
    rsw_worlds: Res<Assets<RswWorld>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    match asset_server.recursive_dependency_load_state(loading_world.world.id()) {
        RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading => {}
        RecursiveDependencyLoadState::Loaded => {
            let Some(world) = rsw_worlds.get(loading_world.world.id()) else {
                unreachable!("Loaded RswWorld must be valid.");
            };
            scene_spawner.spawn(world.scene.clone());
            commands.remove_resource::<LoadingWorld>();
            if let Some(previous_world) = previous_world {
                commands.entity(*previous_world).despawn();
            }
        }
        RecursiveDependencyLoadState::Failed(err) => {
            error!("{err}");
            commands.write_message(AppExit::from_code(1));
        }
    }
}
