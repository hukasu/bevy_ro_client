use bevy::{
    app::{Plugin, Update},
    pbr::DirectionalLight,
    prelude::{
        resource_changed, Children, Commands, Entity, Event, IntoSystemConfigs, LightGizmoColor,
        Local, OnAdd, Query, ReflectResource, Res, Resource, ShowLightGizmo, Trigger, With,
    },
    reflect::Reflect,
};

use crate::assets::rsw;

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
                    // wait_for_world_to_load.run_if(resource_exists::<WorldInLoading>),
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
    environmental_lights: Query<Entity, With<rsw::EnvironmentalLight>>,
) {
    if rsw_debug.show_point_lights {
        for light in environmental_lights.iter() {
            commands.entity(light).insert(ShowLightGizmo::default());
        }
    } else {
        for light in environmental_lights.iter() {
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
    rsws: Query<&Children, With<rsw::World>>,
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

fn world_load(_trigger: Trigger<OnAdd, rsw::World>, mut commands: Commands) {
    commands.trigger(TogglePointLightsDebug);
    commands.trigger(ToggleDirectionalLightDebug);
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct RswDebug {
    show_point_lights: bool,
    show_directional_light: bool,
}

#[derive(Debug, Event)]
struct TogglePointLightsDebug;

#[derive(Debug, Event)]
struct ToggleDirectionalLightDebug;
