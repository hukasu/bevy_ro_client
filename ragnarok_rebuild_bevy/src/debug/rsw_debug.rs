use bevy::{
    app::{Plugin, Update},
    pbr::DirectionalLight,
    prelude::{
        resource_changed, Children, Commands, Entity, Event, HierarchyQueryExt, IntoSystemConfigs,
        LightGizmoColor, Local, Query, ReflectResource, Res, Resource, ShowLightGizmo, Trigger,
        With,
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
    rsws: Query<Entity, With<rsw::World>>,
) {
    if *prev != rsw_debug.show_point_lights {
        *prev = rsw_debug.show_point_lights;

        let Ok(world) = rsws.get_single() else {
            bevy::log::error!("There were an error getting a single World.");
            return;
        };
        commands.trigger_targets(TogglePointLightsDebug, world);
    }
}

fn toggle_point_lights(
    trigger: Trigger<TogglePointLightsDebug>,
    mut commands: Commands,
    rsw_debug: Res<RswDebug>,
    children: Query<&Children>,
    environmental_lights: Query<Entity, With<rsw::EnvironmentalLight>>,
) {
    for light in children
        .iter_descendants(trigger.entity())
        .filter(|child| environmental_lights.contains(*child))
    {
        if rsw_debug.show_point_lights {
            commands.entity(light).insert(ShowLightGizmo::default());
        } else {
            commands.entity(light).remove::<ShowLightGizmo>();
        }
    }
}

fn directional_light_debug_changed(
    mut commands: Commands,
    mut prev: Local<bool>,
    rsws: Query<Entity, With<rsw::World>>,
    rsw_debug: Res<RswDebug>,
) {
    if *prev != rsw_debug.show_directional_light {
        *prev = rsw_debug.show_directional_light;

        let Ok(world) = rsws.get_single() else {
            bevy::log::error!("There were an error getting a single World.");
            return;
        };
        commands.trigger_targets(ToggleDirectionalLightDebug, world);
    }
}

fn toggle_directional_light(
    trigger: Trigger<ToggleDirectionalLightDebug>,
    mut commands: Commands,
    rsw_debug: Res<RswDebug>,
    rsws: Query<&Children, With<rsw::World>>,
    directional_lights: Query<Entity, With<DirectionalLight>>,
) {
    let Ok(world) = rsws.get(trigger.entity()) else {
        bevy::log::error!("World does not have children directional lights to toggle.");
        return;
    };

    for light in world.iter().copied() {
        if let Ok(directional_light) = directional_lights.get(light) {
            if rsw_debug.show_directional_light {
                commands.entity(directional_light).insert(ShowLightGizmo {
                    color: Some(LightGizmoColor::MatchLightColor),
                });
            } else {
                commands
                    .entity(directional_light)
                    .remove::<ShowLightGizmo>();
            }
        }
    }
}

fn world_load(trigger: Trigger<rsw::WorldLoaded>, mut commands: Commands) {
    commands.trigger_targets(TogglePointLightsDebug, trigger.entity());
    commands.trigger_targets(ToggleDirectionalLightDebug, trigger.entity());
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
