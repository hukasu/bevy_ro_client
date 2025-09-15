use bevy::{
    app::{Startup, Update},
    audio::SpatialListener,
    ecs::schedule::common_conditions::{not, resource_exists},
    math::bounding::RayCast3d,
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::bloom::Bloom,
    prelude::{Camera, Camera3d, Commands, GlobalTransform, IntoScheduleConfigs, Query, With},
    render::view::Hdr,
    window::{CursorOptions, PrimaryWindow, Window},
};
use ragnarok_rebuild_bevy::assets::gat::TileRayCast;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Systems
            .add_systems(Startup, create_camera)
            .add_systems(Update, cast_ray_to_ground.run_if(is_mouse_free))
            .add_systems(
                Update,
                drop_ray_to_ground
                    .run_if(resource_exists::<TileRayCast>)
                    .run_if(not(is_mouse_free)),
            );
    }
}

fn create_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            ..Default::default()
        },
        Hdr,
        Atmosphere::EARTH,
        AtmosphereSettings {
            scene_units_to_m: 5.,
            ..Default::default()
        },
        Bloom::NATURAL,
        SpatialListener::default(),
    ));
}

fn is_mouse_free(windows: Query<&CursorOptions, With<PrimaryWindow>>) -> bool {
    if let Ok(cursor_option) = windows.single() {
        matches!(cursor_option.grab_mode, bevy::window::CursorGrabMode::None)
    } else {
        // If there is no window, there is no mouse
        false
    }
}

fn cast_ray_to_ground(
    mut commands: Commands,
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(primary_window) = primary_windows.single() else {
        bevy::log::error!("There are none or more than one PrimaryWindow spawned.");
        return;
    };

    let Ok((camera, camera_transform)) = cameras.single() else {
        bevy::log::error!("There are none or more than one PrimCameraaryWindow spawned.");
        return;
    };

    let Some(cursor_position) = primary_window.cursor_position() else {
        return;
    };

    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Ray is in world space
    let ray_cast = RayCast3d::from_ray(ray, 100.);

    commands.insert_resource(TileRayCast(ray_cast));
}

fn drop_ray_to_ground(mut commands: Commands) {
    commands.remove_resource::<TileRayCast>();
}
