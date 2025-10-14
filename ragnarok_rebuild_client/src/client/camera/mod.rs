use bevy::{
    app::Startup,
    audio::SpatialListener,
    camera::visibility::Visibility,
    ecs::{hierarchy::ChildOf, name::Name, relationship::Relationship},
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::bloom::Bloom,
    prelude::{Camera, Camera3d, Commands},
    render::view::Hdr,
    transform::components::Transform,
};
use bevy_ragnarok_camera::{
    CameraOfOrbitalCamera, OrbitalCameraLimits, OrbitalCameraSettings, TrackedEntity,
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Systems
            .add_systems(Startup, create_camera);
    }
}

fn create_camera(mut commands: Commands) {
    let orbital_camera = commands
        .spawn((
            Name::new("OrbitalCamera"),
            OrbitalCameraSettings {
                pitch: (-45.0f32).to_radians(),
                yaw: 0.0f32.to_radians(),
                zoom: 15.0f32,
            },
            OrbitalCameraLimits {
                pitch_range: (-55.0f32).to_radians()..(-35.0f32).to_radians(),
                zoom_range: 15.0..22.5,
            },
            Transform::default(),
            Visibility::default(),
        ))
        .id();

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
        ChildOf(orbital_camera),
        <CameraOfOrbitalCamera as Relationship>::from(orbital_camera),
    ));

    commands.spawn((
        Name::new("Dummy"),
        Transform::default(),
        Visibility::default(),
        <TrackedEntity as Relationship>::from(orbital_camera),
    ));
}
