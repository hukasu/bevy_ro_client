//! Plugin for dealing with Rsw loading

#[cfg(debug_assertions)]
use bevy::state::state::State;
use bevy::{
    app::{AppExit, PreUpdate},
    asset::{AssetServer, Handle, RecursiveDependencyLoadState},
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::Children,
        lifecycle::{Add, Remove},
        name::NameOrEntity,
        observer::On,
        query::With,
        relationship::RelationshipTarget,
        resource::Resource,
        schedule::{IntoScheduleConfigs, SystemSet},
        system::{Commands, Query, Res, ResMut, Single},
    },
    log::{debug, error, trace},
    scene::{Scene, SceneSpawner},
    state::{app::AppExtStates, commands::CommandsStatesExt, condition::in_state, state::OnEnter},
};
use bevy_ragnarok_rsw::{
    relationships::{ModelsOfWorld, WorldOfModels},
    AnimatedProp, World,
};

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
        app.configure_sets(
            OnEnter(MapChangeStates::LoadingModels),
            WorldSystems::Loading,
        );
        app.configure_sets(OnEnter(MapChangeStates::Loaded), WorldSystems::Cleanup);

        app.add_systems(
            PreUpdate,
            (
                wait_scene.run_if(in_state(MapChangeStates::LoadingAsset)),
                wait_model_scene.run_if(in_state(MapChangeStates::LoadingModels)),
            )
                .in_set(WorldSystems::Loading),
        );
        app.add_systems(
            OnEnter(MapChangeStates::LoadingModels),
            load_models.in_set(WorldSystems::Loading),
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
    /// Runs in [`PreUpdate`] and [`OnEnter<MapChangeStates>`] of [`MapChangeStates::LoadingModels`].
    Loading,
    /// Systems that clean up resources used to load a map.
    /// Runs in [`OnEnter<MapChangeStates>`] of [`MapChangeStates::Loaded`].
    Cleanup,
}

/// [`Handle<Scene>`] of the loading map
#[derive(Resource)]
struct MapChangeScene(Handle<Scene>);

/// [`Handle<Scene>`] of the loading animated prop
#[derive(Component)]
struct LoadingModel(Handle<Scene>);

/// Starts the process of loading a new map
fn map_change(map_change: On<ChangeMap>, mut commands: Commands, asset_server: Res<AssetServer>) {
    let new_map = &*map_change.map;
    trace!("Starting loading of {new_map} map.");

    let map: Handle<Scene> = asset_server.load(format!("data/{new_map}#Scene"));

    commands.set_state(MapChangeStates::LoadingAsset);
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
            commands.remove_resource::<MapChangeScene>();
            commands.set_state(MapChangeStates::LoadingScene);
        }
        RecursiveDependencyLoadState::Failed(err) => {
            error!("{err}");
            commands.write_message(AppExit::from_code(1));
        }
    }
}

/// Load models of [`World`]
fn load_models(
    mut commands: Commands,
    world: Single<(NameOrEntity, &WorldOfModels), With<World>>,
    children: Query<(NameOrEntity, &Children), With<ModelsOfWorld>>,
    animated_props: Query<&AnimatedProp>,
    asset_server: Res<AssetServer>,
) {
    let (world, world_of_models) = world.into_inner();

    let Ok((models, children)) = children.get(*world_of_models.collection()) else {
        debug!("{world} does not have animated props.");
        return;
    };

    for child in children {
        let Ok(animated_prop) = animated_props.get(*child) else {
            unreachable!("All children of {models} must be AnimatedProp.");
        };

        commands.entity(*child).insert(LoadingModel(
            asset_server.load(format!("data/model/{}#Scene", animated_prop.prop_path)),
        ));
    }
}

fn wait_model_scene(
    mut commands: Commands,
    models: Query<(NameOrEntity, &LoadingModel), With<AnimatedProp>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if models.is_empty() {
        commands.set_state(MapChangeStates::Loaded);
        return;
    }

    for (model, loading_model) in models {
        match asset_server.get_recursive_dependency_load_state(loading_model.0.id()) {
            Some(
                RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading,
            ) => (),
            Some(RecursiveDependencyLoadState::Loaded) => {
                commands.entity(model.entity).remove::<LoadingModel>();
                scene_spawner.spawn_as_child(loading_model.0.clone(), model.entity);
            }
            Some(RecursiveDependencyLoadState::Failed(err)) => {
                commands.entity(model.entity).remove::<LoadingModel>();
                error!("Dependecies of {model} failed to load: {err}");
            }
            None => {
                unreachable!("All model scene handles must be valid.")
            }
        }
    }
}

fn new_world_spawned(
    event: On<Add, World>,
    mut commands: Commands,
    worlds: Query<NameOrEntity, With<World>>,
    game: Single<Entity, With<Game>>,
    #[cfg(debug_assertions)] map_change_states: Res<State<MapChangeStates>>,
) {
    #[cfg(debug_assertions)]
    debug_assert_eq!(*map_change_states.get(), MapChangeStates::LoadingScene);

    let world = event.entity;
    let Ok(name_or_entity) = worlds.get(world) else {
        unreachable!("Infallible query.");
    };

    trace!("New World {name_or_entity} spawned.");
    commands.entity(world).insert(WorldOfGame(*game));
    commands.set_state(MapChangeStates::LoadingModels);
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
