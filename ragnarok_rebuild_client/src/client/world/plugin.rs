//! Plugin for dealing with Rsw loading

use bevy::{
    app::{AppExit, PreUpdate},
    asset::{AssetServer, Handle, RecursiveDependencyLoadState},
    ecs::{
        entity::Entity,
        lifecycle::{Add, Remove},
        name::NameOrEntity,
        observer::On,
        query::With,
        resource::Resource,
        schedule::{IntoScheduleConfigs, SystemSet},
        system::{Commands, Query, Res, ResMut, Single},
    },
    log::{error, trace},
    scene::{Scene, SceneSpawner},
    state::{app::AppExtStates, commands::CommandsStatesExt, condition::in_state, state::OnEnter},
};
use bevy_ragnarok_rsw::World;

use crate::client::{
    world::{ChangeMap, MapChangeStates, WorldOfGame},
    Game,
};

/// World plugin
///
/// Registers [`MapChangeStates`], and sets up systems for map change.
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.init_state::<MapChangeStates>();

        app.configure_sets(PreUpdate, WorldSystems::Loading);
        app.configure_sets(OnEnter(MapChangeStates::Loaded), WorldSystems::Cleanup);

        app.add_systems(
            PreUpdate,
            wait_scene
                .run_if(in_state(MapChangeStates::Loading))
                .in_set(WorldSystems::Loading),
        );

        app.add_observer(map_change);
        app.add_observer(new_world_spawned);
        app.add_observer(despawn_old_world);
    }
}

/// [`SystemSet`] in which world systems will run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum WorldSystems {
    /// Systems that deal with the loading of a map.
    /// Runs in [`PreUpdate`].
    Loading,
    /// Systems that clean up resources used to load a map.
    /// Runs in [`OnEnter<MapChangeStates>`] of [`MapChangeStates::Loaded`].
    Cleanup,
}

/// [`InstanceId`] of the loading map
#[derive(Resource)]
struct MapChangeScene(Handle<Scene>);

/// Starts the process of loading a new map
fn map_change(map_change: On<ChangeMap>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let new_map = &*map_change.map;
    trace!("Starting loading of {new_map} map.");

    let map: Handle<Scene> = asset_server.load(format!("data/{new_map}#Scene"));

    commands.set_state(MapChangeStates::Loading);
    commands.insert_resource(MapChangeScene(map));
}

/// Wait for [`Scene`] of [`RswAsset`](bevy_ragnarok_rsw::assets::RswAsset) to finish loading
fn wait_scene(
    mut commands: Commands,
    game: Single<Entity, With<Game>>,
    map_change_scene: Res<MapChangeScene>,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_server: Res<AssetServer>,
) {
    match asset_server.recursive_dependency_load_state(map_change_scene.0.id()) {
        RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading => (),
        RecursiveDependencyLoadState::Loaded => {
            scene_spawner.spawn_as_child(map_change_scene.0.clone(), *game);
            commands.set_state(MapChangeStates::Loaded);
        }
        RecursiveDependencyLoadState::Failed(err) => {
            error!("{err}");
            commands.write_message(AppExit::from_code(1));
        }
    }
}

fn new_world_spawned(
    event: On<Add, World>,
    mut commands: Commands,
    worlds: Query<NameOrEntity, With<World>>,
    game: Single<Entity, With<Game>>,
) {
    let world = event.entity;
    let Ok(name_or_entity) = worlds.get(world) else {
        unreachable!("Infallible query.");
    };

    trace!("New World {name_or_entity} spawned.");
    commands.entity(world).insert(WorldOfGame(*game));
    commands.set_state(MapChangeStates::Loaded);
}

fn despawn_old_world(
    event: On<Remove, WorldOfGame>,
    mut commands: Commands,
    worlds: Query<NameOrEntity, With<World>>,
) {
    let world = event.entity;
    let Ok(name_or_entity) = worlds.get(world) else {
        unreachable!("Infallible query.");
    };

    trace!("Despawning {name_or_entity}.");
    commands.entity(event.entity).despawn();
}
