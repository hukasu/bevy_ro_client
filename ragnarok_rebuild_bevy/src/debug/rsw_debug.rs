use bevy::{
    app::{Plugin, Update},
    pbr::DirectionalLight,
    prelude::{
        resource_changed, resource_exists, Children, Commands, Entity, Event, IntoSystemConfigs,
        LightGizmoColor, Local, OnAdd, Query, ReflectResource, Res, Resource, ShowLightGizmo,
        Trigger, With,
    },
    reflect::Reflect,
    scene::{SceneInstance, SceneSpawner},
};

use crate::{assets::rsw, world};

pub struct RswDebugPlugin;

impl Plugin for RswDebugPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Resource
            .register_type::<RswDebug>()
            .init_resource::<RswDebug>()
            // System
            .add_systems(
                Update,
                (
                    point_light_debug_changed.run_if(resource_changed::<RswDebug>),
                    directional_light_debug_changed.run_if(resource_changed::<RswDebug>),
                    wait_for_world_to_load.run_if(resource_exists::<WorldInLoading>),
                ),
            )
            // Observers
            .observe(toggle_point_lights)
            .observe(toggle_directional_light)
            .observe(world_load);
    }
}

fn point_light_debug_changed(
    mut commands: Commands,
    mut prev: Local<bool>,
    rsw_debug: Res<RswDebug>,
) {
    if *prev != rsw_debug.show_point_lights {
        *prev = rsw_debug.show_point_lights;
        commands.trigger(TogglePointLightsDebug)
    }
}

fn toggle_point_lights(
    _trigger: Trigger<TogglePointLightsDebug>,
    mut commands: Commands,
    rsw_debug: Res<RswDebug>,
    environmental_lights: Query<Entity, With<rsw::EnvironmentalLights>>,
    children: Query<&Children>,
) {
    let Ok(environmental_lights) = environmental_lights.get_single() else {
        bevy::log::error!(
            "There were an error getting a single Environmental lights container from the world."
        );
        return;
    };

    let Ok(children) = children.get(environmental_lights) else {
        bevy::log::trace!("Environmental Lights container existed but had no children.");
        return;
    };

    if rsw_debug.show_point_lights {
        for light in children.iter().copied() {
            commands.entity(light).insert(ShowLightGizmo::default());
        }
    } else {
        for light in children.iter().copied() {
            commands.entity(light).remove::<ShowLightGizmo>();
        }
    }
}

fn directional_light_debug_changed(
    mut commands: Commands,
    mut prev: Local<bool>,
    rsw_debug: Res<RswDebug>,
) {
    if *prev != rsw_debug.show_directional_light {
        *prev = rsw_debug.show_directional_light;
        commands.trigger(ToggleDirectionalLightDebug)
    }
}

fn toggle_directional_light(
    _trigger: Trigger<ToggleDirectionalLightDebug>,
    mut commands: Commands,
    rsw_debug: Res<RswDebug>,
    rsws: Query<&Children, With<world::World>>,
    directional_lights: Query<Entity, With<DirectionalLight>>,
) {
    let Ok(world) = rsws.get_single() else {
        bevy::log::error!("There were an error getting a single World.");
        return;
    };

    if rsw_debug.show_directional_light {
        for light in world.iter().copied() {
            if let Ok(directional_light) = directional_lights.get(light) {
                commands.entity(directional_light).insert(ShowLightGizmo {
                    color: Some(LightGizmoColor::MatchLightColor),
                });
            }
        }
    } else {
        for light in world.iter().copied() {
            if let Ok(directional_light) = directional_lights.get(light) {
                commands
                    .entity(directional_light)
                    .remove::<ShowLightGizmo>();
            }
        }
    }
}

fn world_load(trigger: Trigger<OnAdd, world::World>, mut commands: Commands) {
    let entity = trigger.entity();

    commands.insert_resource(WorldInLoading { entity });
}

fn wait_for_world_to_load(
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    world_in_loading: Res<WorldInLoading>,
    scenes: Query<&SceneInstance>,
) {
    let Ok(scene) = scenes.get(world_in_loading.entity) else {
        bevy::log::error!("World in loading does not have SceneInstance.");
        commands.remove_resource::<WorldInLoading>();
        return;
    };

    if scene_spawner.instance_is_ready(**scene) {
        commands.remove_resource::<WorldInLoading>();
        commands.trigger(TogglePointLightsDebug);
        commands.trigger(ToggleDirectionalLightDebug);
    }
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct RswDebug {
    show_point_lights: bool,
    show_directional_light: bool,
}

#[derive(Debug, Resource)]
struct WorldInLoading {
    entity: Entity,
}

#[derive(Debug, Event)]
struct TogglePointLightsDebug;

#[derive(Debug, Event)]
struct ToggleDirectionalLightDebug;
