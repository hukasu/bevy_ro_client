use bevy::{
    app::{PostUpdate, Update},
    color::palettes,
    math::{Dir3, EulerRot, Quat, Vec2, Vec3},
    pbr::{DirectionalLight, PointLight},
    prelude::{
        Children, Entity, Gizmos, GlobalTransform, IntoSystemConfigs, Query, ReflectResource, Res,
        Resource, Transform, With,
    },
    reflect::Reflect,
    render::view::VisibilitySystems,
};

use crate::{assets::rsw, helper::AabbExt};

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct RswDebug {
    show_directional_light: bool,
    show_point_lights: bool,
    show_sounds: bool,
    show_effects: bool,
    show_quad_tree: bool,
    show_quad_tree_level: u8,
}

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Resource
            .register_type::<RswDebug>()
            .init_resource::<RswDebug>()
            // System
            .add_systems(
                Update,
                (
                    directional_light_debug.run_if(directional_light_debug_condition),
                    point_light_debug.run_if(point_light_debug_condition),
                    effect_debug.run_if(effect_debug_condition),
                ),
            )
            .add_systems(
                PostUpdate,
                show_rsw_quad_tree
                    .run_if(show_rsw_aabb_condition)
                    .after(VisibilitySystems::CheckVisibility),
            );

        #[cfg(feature = "audio")]
        app.add_systems(Update, sound_debug.run_if(sound_debug_condition));
    }
}

fn directional_light_debug_condition(rsw_debug: Res<RswDebug>) -> bool {
    rsw_debug.show_directional_light
}

fn directional_light_debug(
    mut gizmos: Gizmos,
    worlds: Query<Entity, With<rsw::World>>,
    children: Query<&Children>,
    directional_lights: Query<(&GlobalTransform, &DirectionalLight)>,
) {
    const DIRECTIONAL_LIGHT_GIZMO_LENGHT: f32 = 5.;

    let Ok(world) = worlds.get_single() else {
        return;
    };

    let Ok(world_children) = children.get(world) else {
        bevy::log::error!("Can't show directional light gizmos because World has no children.");
        return;
    };

    let Some((directional_light_pos, directional_light)) = world_children
        .iter()
        .find_map(|child| directional_lights.get(*child).ok())
    else {
        return;
    };

    let color = directional_light.color;
    let (_, rotation, translation) = directional_light_pos.to_scale_rotation_translation();
    gizmos.rect(
        translation,
        rotation,
        Vec2::splat(DIRECTIONAL_LIGHT_GIZMO_LENGHT / 2.),
        color,
    );
    for x in [
        -DIRECTIONAL_LIGHT_GIZMO_LENGHT,
        0.,
        DIRECTIONAL_LIGHT_GIZMO_LENGHT,
    ] {
        for y in [
            -DIRECTIONAL_LIGHT_GIZMO_LENGHT,
            0.,
            DIRECTIONAL_LIGHT_GIZMO_LENGHT,
        ] {
            gizmos.arrow(
                directional_light_pos.transform_point(Vec3::new(x, y, 0.)),
                directional_light_pos.transform_point(Vec3::new(
                    x,
                    y,
                    -DIRECTIONAL_LIGHT_GIZMO_LENGHT,
                )),
                color,
            );
        }
    }
}

fn point_light_debug_condition(rsw_debug: Res<RswDebug>) -> bool {
    rsw_debug.show_point_lights
}

fn point_light_debug(
    mut gizmos: Gizmos,
    worlds: Query<(Entity, &rsw::World)>,
    children: Query<&Children>,
    lights: Query<(&GlobalTransform, &PointLight)>,
) {
    const POINT_LIGHT_GIZMO_LENGHT: f32 = 1.;
    const POINT_LIGHT_RANGE_THRESHOLD: f32 = 5.;

    let Ok((world, world_info)) = worlds.get_single() else {
        return;
    };
    if !world_info.has_lights {
        return;
    }

    let Ok(world_children) = children.get(world) else {
        bevy::log::error!("Can't show point light gizmos because World has no children.");
        return;
    };

    let Some(lights_container) = world_children.iter().find_map(|child| {
        let Ok(child_children) = children.get(*child) else {
            return None;
        };
        if lights.contains(child_children[0]) {
            Some(child_children)
        } else {
            None
        }
    }) else {
        return;
    };

    for (light_pos, light_properties) in lights.iter_many(lights_container) {
        let color = light_properties.color;
        let translation = light_pos.translation();

        gizmos.sphere(translation, Quat::default(), light_properties.range, color);
        let scale_factor = if light_properties.range < POINT_LIGHT_RANGE_THRESHOLD {
            light_properties.range / POINT_LIGHT_RANGE_THRESHOLD
        } else {
            1.
        };
        gizmos.sphere(
            translation,
            Quat::default(),
            POINT_LIGHT_GIZMO_LENGHT * scale_factor,
            color,
        );
        // Poles
        let pole_offset = Vec3::new(0., POINT_LIGHT_GIZMO_LENGHT, 0.);
        gizmos.line(
            translation + pole_offset * scale_factor,
            translation + (pole_offset * 2.) * scale_factor,
            color,
        );
        gizmos.line(
            translation + -pole_offset * scale_factor,
            translation + (-pole_offset * 2.) * scale_factor,
            color,
        );

        let equator_offset = Vec3::new(POINT_LIGHT_GIZMO_LENGHT, 0., 0.);
        for y in [0., 1., 2., 3., 4., 5., 6., 7., 8.] {
            for z in [-1., 0., 1.] {
                let transform = Transform::from_rotation(Quat::from_euler(
                    EulerRot::YZX,
                    std::f32::consts::FRAC_PI_4 * y,
                    std::f32::consts::FRAC_PI_4 * z,
                    0.,
                ));
                gizmos.line(
                    translation + transform.transform_point(equator_offset) * scale_factor,
                    translation + transform.transform_point(equator_offset * 2.) * scale_factor,
                    color,
                );
            }
        }
    }
}

#[cfg(feature = "audio")]
fn sound_debug_condition(rsw_debug: Res<RswDebug>) -> bool {
    rsw_debug.show_sounds
}

#[cfg(feature = "audio")]
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
        bevy::log::error!("Can't show sound gizmos because World has no children.");
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
            std::f32::consts::FRAC_PI_2,
            SOUND_GIZMO_RADIUS / 3.,
            sounds_translation,
            Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                -std::f32::consts::FRAC_PI_4,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            std::f32::consts::FRAC_PI_2,
            SOUND_GIZMO_RADIUS * 2. / 3.,
            sounds_translation,
            Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                -std::f32::consts::FRAC_PI_4,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            std::f32::consts::FRAC_PI_2,
            SOUND_GIZMO_RADIUS,
            sounds_translation,
            Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                -std::f32::consts::FRAC_PI_4,
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
            std::f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(-EFFECT_GIZMO_RADIUS, -EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(EulerRot::XYZ, std::f32::consts::FRAC_PI_2, 0., 0.),
            color,
        );
        gizmos.arc_3d(
            std::f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(EFFECT_GIZMO_RADIUS, EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::PI,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            std::f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(-EFFECT_GIZMO_RADIUS, EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2 * 3.,
                0.,
            ),
            color,
        );
        gizmos.arc_3d(
            std::f32::consts::FRAC_PI_2,
            EFFECT_GIZMO_RADIUS,
            translation + Vec3::new(EFFECT_GIZMO_RADIUS, -EFFECT_GIZMO_RADIUS, 0.),
            Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                std::f32::consts::FRAC_PI_2,
                0.,
            ),
            color,
        );
        gizmos.circle(translation, Dir3::NEG_Z, EFFECT_GIZMO_RADIUS, color);
    }
}

fn show_rsw_quad_tree(
    mut gizmos: Gizmos,
    rsw_debug: Res<RswDebug>,
    rsws: Query<(&super::components::World, &GlobalTransform)>,
) {
    for (world, rsw_transform) in rsws.iter() {
        for node_index in world
            .quad_tree
            .iter_indexes()
            .filter(|node| node.depth() == usize::from(rsw_debug.show_quad_tree_level))
        {
            let aabb = world.quad_tree[node_index];

            gizmos.cuboid(
                aabb.compute_global_transform(*rsw_transform),
                palettes::tailwind::BLUE_400,
            );
        }
    }
}

fn show_rsw_aabb_condition(rsw_debug: Res<RswDebug>) -> bool {
    rsw_debug.show_quad_tree
        & (usize::from(rsw_debug.show_quad_tree_level) <= rsw::quad_tree::QuadTree::MAX_DEPTH)
}
