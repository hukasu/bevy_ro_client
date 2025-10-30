//! Plugin for dealing with Rsw loading

#[cfg(debug_assertions)]
use bevy::state::state::State;
use bevy::{
    animation::AnimationPlayer,
    app::{AppExit, PreUpdate},
    asset::{AssetServer, Handle, RecursiveDependencyLoadState},
    camera::visibility::Visibility,
    ecs::{
        component::Component,
        entity::Entity,
        hierarchy::{ChildOf, Children},
        lifecycle::{Add, Remove},
        name::NameOrEntity,
        observer::On,
        query::With,
        relationship::RelationshipTarget,
        resource::Resource,
        schedule::{IntoScheduleConfigs, SystemSet},
        system::{Commands, Populated, Query, Res, ResMut, Single},
    },
    log::{debug, error, trace},
    math::Vec3,
    scene::{Scene, SceneSpawner},
    state::{app::AppExtStates, commands::CommandsStatesExt, condition::in_state, state::OnEnter},
    transform::components::Transform,
};
use bevy_ragnarok_gnd::Ground as GndGround;
use bevy_ragnarok_rsm::Model;
use bevy_ragnarok_rsw::{
    relationships::{
        AltitudeOfWorld, GroundOfWorld, ModelsOfWorld, WorldOfAltitude, WorldOfGround,
        WorldOfModels,
    },
    Altitude, AnimatedProp, Ground, World,
};
use bevy_ragnarok_water_plane::WaterPlaneBuilder;

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
                wait_ground_scene.run_if(in_state(MapChangeStates::LoadingGround)),
                wait_altitude_scene.run_if(in_state(MapChangeStates::LoadingAltitude)),
                wait_model_scene.run_if(in_state(MapChangeStates::LoadingModels)),
            )
                .in_set(WorldSystems::Loading),
        );
        app.add_systems(
            OnEnter(MapChangeStates::LoadingGround),
            load_ground.in_set(WorldSystems::Loading),
        );
        app.add_systems(
            OnEnter(MapChangeStates::LoadingAltitude),
            load_altitude.in_set(WorldSystems::Loading),
        );
        app.add_systems(
            OnEnter(MapChangeStates::LoadingRswWaterPlane),
            load_rsw_water_plane.in_set(WorldSystems::Loading),
        );
        app.add_systems(
            OnEnter(MapChangeStates::LoadingModels),
            load_models.in_set(WorldSystems::Loading),
        );
        app.add_systems(
            OnEnter(MapChangeStates::Loaded),
            (start_animations, update_game_transform).in_set(WorldSystems::Cleanup),
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

/// [`Handle<Scene>`] of the loading ground
#[derive(Component)]
struct LoadingGround(Handle<Scene>);

/// [`Handle<Scene>`] of the loading altitude tiles
#[derive(Component)]
struct LoadingAltitude(Handle<Scene>);

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

/// Load ground of [`World`]
fn load_ground(
    mut commands: Commands,
    world: Single<(NameOrEntity, &WorldOfGround), With<World>>,
    children: Query<(NameOrEntity, &Ground), With<GroundOfWorld>>,
    asset_server: Res<AssetServer>,
) {
    let (world, world_of_models) = world.into_inner();

    let Ok((ground, ground_path)) = children.get(*world_of_models.collection()) else {
        error!("{world} does not ground.");
        commands.write_message(AppExit::from_code(1));
        return;
    };

    commands.entity(ground.entity).insert(LoadingGround(
        asset_server.load(format!("data/{}#Scene", ground_path.ground_path)),
    ));
}

/// Load altitude tiles of [`World`]
fn load_altitude(
    mut commands: Commands,
    world: Single<(NameOrEntity, &WorldOfAltitude), With<World>>,
    children: Query<(NameOrEntity, &Altitude), With<AltitudeOfWorld>>,
    asset_server: Res<AssetServer>,
) {
    let (world, world_of_altitude) = world.into_inner();

    let Ok((altitude, Altitude { altitude_path })) = children.get(*world_of_altitude.collection())
    else {
        error!("{world} does not have altitude.");
        commands.write_message(AppExit::from_code(1));
        return;
    };

    commands
        .entity(altitude.entity)
        .insert(LoadingAltitude(asset_server.load_with_settings(
            format!("data/{}#Scene", altitude_path),
            |settings: &mut f32| {
                *settings = 5.;
            },
        )));
}
/// Load [`WaterPlane`](bevy_ragnarok_water_plane::WaterPlane) of [`World`]
fn load_rsw_water_plane(
    mut commands: Commands,
    world: Single<(NameOrEntity, &World), With<World>>,
    ground: Single<&GndGround>,
) {
    let (world_entity, world) = world.into_inner();

    if let Some(water_plane) = &world.water_plane {
        commands.spawn((
            ChildOf(world_entity.entity),
            WaterPlaneBuilder {
                width: ground.width - 4,
                height: ground.height - 4,
                water_plane: water_plane.clone(),
            },
            Transform::from_scale(Vec3::new(ground.scale, 1., ground.scale)),
            Visibility::default(),
        ));
    }

    commands.set_state(MapChangeStates::LoadingModels);
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

fn wait_ground_scene(
    mut commands: Commands,
    grounds: Query<(NameOrEntity, &LoadingGround), With<Ground>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if grounds.is_empty() {
        commands.set_state(MapChangeStates::LoadingAltitude);
        return;
    }

    for (ground, loading_ground) in grounds {
        match asset_server.get_recursive_dependency_load_state(loading_ground.0.id()) {
            Some(
                RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading,
            ) => (),
            Some(RecursiveDependencyLoadState::Loaded) => {
                commands.entity(ground.entity).remove::<LoadingGround>();
                scene_spawner.spawn_as_child(loading_ground.0.clone(), ground.entity);
            }
            Some(RecursiveDependencyLoadState::Failed(err)) => {
                commands.entity(ground.entity).remove::<LoadingGround>();
                error!("Dependencies of {ground} failed to load: {err}");
            }
            None => {
                unreachable!("All model scene handles must be valid.")
            }
        }
    }
}

fn wait_altitude_scene(
    mut commands: Commands,
    altitudes: Query<(NameOrEntity, &LoadingAltitude), With<Altitude>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if altitudes.is_empty() {
        commands.set_state(MapChangeStates::LoadingRswWaterPlane);
        return;
    }

    for (altitude, LoadingAltitude(altitude_handle)) in altitudes {
        match asset_server.get_recursive_dependency_load_state(altitude_handle.id()) {
            Some(
                RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading,
            ) => (),
            Some(RecursiveDependencyLoadState::Loaded) => {
                commands.entity(altitude.entity).remove::<LoadingAltitude>();
                scene_spawner.spawn_as_child(altitude_handle.clone(), altitude.entity);
            }
            Some(RecursiveDependencyLoadState::Failed(err)) => {
                commands.entity(altitude.entity).remove::<LoadingAltitude>();
                error!("Dependencies of {altitude} failed to load: {err}");
            }
            None => {
                unreachable!("All altitude scene handles must be valid.")
            }
        }
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
                error!("Dependencies of {model} failed to load: {err}");
            }
            None => {
                unreachable!("All model scene handles must be valid.")
            }
        }
    }
}

fn start_animations(
    animated_props: Populated<(NameOrEntity, &AnimatedProp, &Children)>,
    mut models: Query<(&mut AnimationPlayer, &Model)>,
) {
    for (entity, animated_prop, children) in animated_props.into_inner() {
        if animated_prop.animation_type == 0 {
            continue;
        }

        debug_assert_eq!(children.len(), 1);

        let child = children.collection()[0];
        let Ok((mut animation_player, model)) = models.get_mut(child) else {
            error!("{entity}'s child was not a Model.");
            continue;
        };

        if let Some(animation_id) = &model.animation {
            let animation = animation_player.play(animation_id.animation_node_index);
            if animated_prop.animation_type == 2 {
                animation.repeat();
            }
            animation.set_speed(animated_prop.animation_speed);
        }
    }
}

fn update_game_transform(mut game: Single<&mut Transform, With<Game>>, ground: Single<&GndGround>) {
    let scale = 2. / ground.scale;
    game.scale = Vec3::new(scale, -scale, -scale);
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
    commands.set_state(MapChangeStates::LoadingGround);
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
