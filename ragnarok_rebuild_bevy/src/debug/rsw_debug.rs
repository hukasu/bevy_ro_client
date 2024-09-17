use core::f32;

use bevy::{
    app::{Plugin, Update},
    color::palettes,
    math::{Dir3, Quat, Vec3},
    pbr::DirectionalLight,
    prelude::{
        resource_changed, Children, Commands, Entity, Event, Gizmos, GlobalTransform,
        HierarchyQueryExt, IntoSystemConfigs, LightGizmoColor, Local, Query, ReflectResource, Res,
        Resource, ShowLightGizmo, Trigger, With,
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
                    sound_debug.run_if(sound_debug_condition),
                    effect_debug.run_if(effect_debug_condition),
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
    rsws: Query<(Entity, &rsw::World)>,
) {
    if *prev != rsw_debug.show_point_lights {
        *prev = rsw_debug.show_point_lights;

        let Ok((world, world_info)) = rsws.get_single() else {
            bevy::log::error!("There were an error getting a single World.");
            return;
        };
        if !world_info.has_lights {
            return;
        }
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

fn sound_debug_condition(rsw_debug: Res<RswDebug>) -> bool {
    rsw_debug.show_sounds
}

fn sound_debug(
    mut gizmos: Gizmos,
    worlds: Query<(Entity, &rsw::World)>,
    children: Query<&Children>,
    sounds: Query<(&GlobalTransform, &rsw::EnvironmentalSound)>,
) {
    const SOUND_GIZMO_RADIUS: f32 = 5.;

    let Ok((world, world_info)) = worlds.get_single() else {
        return;
    };
    if !world_info.has_sounds {
        return;
    }

    let Ok(world_children) = children.get(world) else {
        bevy::log::error!("Can't show effect gizmos because World has no children.");
        return;
    };

    let Some(sounds_container) = world_children.iter().find_map(|child| {
        let Ok(child_children) = children.get(*child) else {
            return None;
        };
        if sounds.contains(child_children[0]) {
            Some(child_children)
        } else {
            None
        }
    }) else {
        return;
    };

    let color = palettes::css::SEA_GREEN;
    for (effect_transform, effect_properties) in sounds.iter_many(sounds_container) {
        let translation = effect_transform.translation();
        let sounds_translation = translation + Vec3::new(-SOUND_GIZMO_RADIUS / 2., 0., 0.);
        gizmos.sphere(
            translation,
            Quat::default(),
            effect_properties.range / 5.,
            color,
        );
        gizmos.arc_3d(
            f32::consts::FRAC_PI_2,
            SOUND_GIZMO_RADIUS / 3.,
            sounds_translation,
            Quat::from_euler(
                bevy::math::EulerRot::XYZ,
                f32::consts::FRAC_PI_2,
                -f32::consts::FRAC_PI_4,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            f32::consts::FRAC_PI_2,
            SOUND_GIZMO_RADIUS * 2. / 3.,
            sounds_translation,
            Quat::from_euler(
                bevy::math::EulerRot::XYZ,
                f32::consts::FRAC_PI_2,
                -f32::consts::FRAC_PI_4,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            f32::consts::FRAC_PI_2,
            SOUND_GIZMO_RADIUS,
            sounds_translation,
            Quat::from_euler(
                bevy::math::EulerRot::XYZ,
                f32::consts::FRAC_PI_2,
                -f32::consts::FRAC_PI_4,
                0.,
            ),
            color,
        );
        gizmos.circle(translation, Dir3::NEG_Z, SOUND_GIZMO_RADIUS, color);
    }
}

fn effect_debug_condition(rsw_debug: Res<RswDebug>) -> bool {
    rsw_debug.show_effects
}

fn effect_debug(
    mut gizmos: Gizmos,
    worlds: Query<(Entity, &rsw::World)>,
    children: Query<&Children>,
    effects: Query<&GlobalTransform, With<rsw::EnvironmentalEffect>>,
) {
    const EFFECT_GIZMO_RADIUS: f32 = 5.;

    let Ok((world, world_info)) = worlds.get_single() else {
        return;
    };
    if !world_info.has_effects {
        return;
    }

    let Ok(world_children) = children.get(world) else {
        bevy::log::error!("Can't show effect gizmos because World has no children.");
        return;
    };

    let Some(effects_container) = world_children.iter().find_map(|child| {
        let Ok(child_children) = children.get(*child) else {
            return None;
        };
        if effects.contains(child_children[0]) {
            Some(child_children)
        } else {
            None
        }
    }) else {
        return;
    };

    let color = palettes::css::SKY_BLUE;
    for effect in effects.iter_many(effects_container) {
        let translation = effect.translation();
        gizmos.arc_3d(
            f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(-EFFECT_GIZMO_RADIUS, -EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(bevy::math::EulerRot::XYZ, f32::consts::FRAC_PI_2, 0., 0.),
            color,
        );
        gizmos.arc_3d(
            f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(EFFECT_GIZMO_RADIUS, EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(
                bevy::math::EulerRot::XYZ,
                f32::consts::FRAC_PI_2,
                f32::consts::PI,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(-EFFECT_GIZMO_RADIUS, EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(
                bevy::math::EulerRot::XYZ,
                f32::consts::FRAC_PI_2,
                f32::consts::FRAC_PI_2 * 3.,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(EFFECT_GIZMO_RADIUS, -EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(
                bevy::math::EulerRot::XYZ,
                f32::consts::FRAC_PI_2,
                f32::consts::FRAC_PI_2,
                0.,
            ),
            color,
        );
        gizmos.circle(translation, Dir3::NEG_Z, EFFECT_GIZMO_RADIUS, color);
    }
}

fn world_load(
    trigger: Trigger<rsw::WorldLoaded>,
    mut commands: Commands,
    worlds: Query<&rsw::World>,
) {
    if worlds
        .get(trigger.entity())
        .ok()
        .filter(|world| world.has_lights)
        .is_some()
    {
        commands.trigger_targets(TogglePointLightsDebug, trigger.entity());
    }
    commands.trigger_targets(ToggleDirectionalLightDebug, trigger.entity());
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct RswDebug {
    show_point_lights: bool,
    show_directional_light: bool,
    show_sounds: bool,
    show_effects: bool,
}

#[derive(Debug, Event)]
struct TogglePointLightsDebug;

#[derive(Debug, Event)]
struct ToggleDirectionalLightDebug;
