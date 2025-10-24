use bevy_app::Update;
use bevy_color::{self, palettes};
use bevy_ecs::{
    event::Event,
    hierarchy::Children,
    observer::On,
    query::With,
    reflect::ReflectResource,
    resource::Resource,
    schedule::{IntoScheduleConfigs, common_conditions::resource_changed},
    system::{Commands, Local, Query, Res, ResMut},
};
use bevy_gizmos::aabb::ShowAabbGizmo;
use bevy_log::debug;
use bevy_reflect::Reflect;

use crate::Cube;

pub(crate) struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Resources
        app.register_type::<GndDebug>().init_resource::<GndDebug>();
        app.add_systems(
            Update,
            trigger_on_changes.run_if(resource_changed::<GndDebug>),
        );
        // Observers
        app.add_observer(toggle_gnd_aabbs);
    }
}

#[derive(Debug, Clone, Copy, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GndDebug {
    show_aabbs: bool,
}

#[derive(Debug, Event)]
pub struct ToggleGndAabbs;

fn toggle_gnd_aabbs(_event: On<ToggleGndAabbs>, mut gat_debug: ResMut<GndDebug>) {
    debug!("Toggling Gnd Aabbs");
    gat_debug.show_aabbs = !gat_debug.show_aabbs;
}

fn enable_gnd_aabbs(mut commands: Commands, cubes: Query<&Children, With<Cube>>) {
    debug!("Enabling Gnd Aabbs");
    let cube_aabb_color = palettes::tailwind::PURPLE_300.into();
    for children in cubes {
        if let Some(child) = children.first() {
            commands.entity(*child).insert(ShowAabbGizmo {
                color: Some(cube_aabb_color),
            });
        }
    }
}

fn disable_gnd_aabbs(mut commands: Commands, cubes: Query<&Children, With<Cube>>) {
    debug!("Disabling Gat Aabbs");
    for child in cubes.iter().flatten() {
        commands.entity(*child).remove::<ShowAabbGizmo>();
    }
}

fn trigger_on_changes(
    mut commands: Commands,
    gnd_debug: Res<GndDebug>,
    mut gnd_debug_cache: Local<GndDebug>,
) {
    if gnd_debug.show_aabbs != gnd_debug_cache.show_aabbs {
        match gnd_debug.show_aabbs {
            true => commands.run_system_cached(enable_gnd_aabbs),
            false => commands.run_system_cached(disable_gnd_aabbs),
        }
    }

    *gnd_debug_cache = *gnd_debug;
}
