use std::f32::consts::TAU;

use bevy::{
    app::Startup,
    audio::SpatialListener,
    camera::visibility::Visibility,
    ecs::{
        entity::Entity, hierarchy::ChildOf, name::Name, observer::On, query::With,
        relationship::Relationship, system::Single,
    },
    input::mouse::MouseButton,
    math::Vec2,
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::bloom::Bloom,
    prelude::{Camera, Camera3d, Commands},
    render::view::Hdr,
    transform::components::Transform,
};
use bevy_enhanced_input::{
    action::Action,
    prelude::{
        ActionOf, Binding, BindingOf, Chord, ContextPriority, Fire, InputAction,
        InputContextAppExt, InputModKeys, ModKeys, Scale,
    },
};
use bevy_ragnarok_camera::{
    CameraOfOrbitalCamera, OrbitalCamera, OrbitalCameraLimits, OrbitalCameraSettings, TrackedEntity,
};

use crate::client::camera::{
    CameraPitch, CameraYaw, CameraZoom, OrbitalCameraPrimaryContext, OrbitalCameraSecondaryContext,
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_input_context::<OrbitalCameraPrimaryContext>();
        app.add_input_context::<OrbitalCameraSecondaryContext>();

        // Systems
        app.add_systems(Startup, setup_orbital_camera);
        app.add_observer(camera_yaw);
        app.add_observer(camera_pitch);
        app.add_observer(camera_zoom);
    }
}

#[derive(InputAction)]
#[action_output(Vec2)]
struct MouseRightAction;

#[derive(InputAction)]
#[action_output(Vec2)]
struct ShiftMouseRightAction;

fn setup_orbital_camera(mut commands: Commands) {
    let orbital_camera = commands
        .spawn((
            Name::new("OrbitalCamera"),
            OrbitalCameraSettings {
                pitch: (-45.0f32).to_radians(),
                yaw: 0.0f32.to_radians(),
                zoom: 15.0f32,
            },
            OrbitalCameraLimits {
                yaw_default: 0.0,
                yaw_range: 0.0..TAU,
                pitch_default: -45.0f32.to_radians(),
                pitch_range: (-55.0f32).to_radians()..(-35.0f32).to_radians(),
                zoom_default: 15.0,
                zoom_range: 15.0..22.5,
            },
            OrbitalCameraPrimaryContext,
            OrbitalCameraSecondaryContext,
            ContextPriority::<OrbitalCameraPrimaryContext>::new(1),
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

    spawn_camera_yaw_action(&mut commands, orbital_camera);
    spawn_camera_pitch_action(&mut commands, orbital_camera);
    spawn_camera_zoom_action(&mut commands, orbital_camera);

    commands.spawn((
        Name::new("Dummy"),
        Transform::default(),
        Visibility::default(),
        <TrackedEntity as Relationship>::from(orbital_camera),
    ));
}

fn camera_yaw(
    event: On<Fire<CameraYaw>>,
    mut orbital_camera: Single<&mut OrbitalCameraSettings, With<OrbitalCamera>>,
) {
    orbital_camera.yaw -= event.value.x;
}

fn camera_pitch(
    event: On<Fire<CameraPitch>>,
    mut orbital_camera: Single<&mut OrbitalCameraSettings, With<OrbitalCamera>>,
) {
    orbital_camera.pitch += event.value.y;
}

fn camera_zoom(
    event: On<Fire<CameraZoom>>,
    mut orbital_camera: Single<&mut OrbitalCameraSettings, With<OrbitalCamera>>,
) {
    orbital_camera.zoom += event.value.y;
}

fn spawn_camera_yaw_action(commands: &mut Commands, camera: Entity) {
    let mouse_right = commands
        .spawn((
            ChildOf(camera),
            ActionOf::<OrbitalCameraPrimaryContext>::new(camera),
            Action::<MouseRightAction>::new(),
        ))
        .id();
    commands.spawn((
        ChildOf(mouse_right),
        <BindingOf as Relationship>::from(mouse_right),
        Binding::from(MouseButton::Right),
    ));

    let camera_yaw = commands
        .spawn((
            ChildOf(camera),
            ActionOf::<OrbitalCameraPrimaryContext>::new(camera),
            Action::<CameraYaw>::new(),
            Scale::splat(1. / 128.),
            Chord::new(vec![mouse_right]),
        ))
        .id();
    commands.spawn((
        ChildOf(camera_yaw),
        <BindingOf as Relationship>::from(camera_yaw),
        Binding::mouse_motion(),
    ));
}

fn spawn_camera_pitch_action(commands: &mut Commands, camera: Entity) {
    let shift_mouse_right = commands
        .spawn((
            ChildOf(camera),
            ActionOf::<OrbitalCameraPrimaryContext>::new(camera),
            Action::<ShiftMouseRightAction>::new(),
        ))
        .id();
    commands.spawn((
        ChildOf(shift_mouse_right),
        <BindingOf as Relationship>::from(shift_mouse_right),
        MouseButton::Right.with_mod_keys(ModKeys::SHIFT),
    ));

    let camera_pith = commands
        .spawn((
            ChildOf(camera),
            ActionOf::<OrbitalCameraPrimaryContext>::new(camera),
            Action::<CameraPitch>::new(),
            Scale::splat(1. / 128.),
            Chord::new(vec![shift_mouse_right]),
        ))
        .id();
    commands.spawn((
        ChildOf(camera_pith),
        <BindingOf as Relationship>::from(camera_pith),
        Binding::mouse_motion(),
    ));
}

fn spawn_camera_zoom_action(commands: &mut Commands, camera: Entity) {
    let camera_zoom = commands
        .spawn((
            ChildOf(camera),
            ActionOf::<OrbitalCameraPrimaryContext>::new(camera),
            Action::<CameraZoom>::new(),
            Scale::splat(1. / 4.),
        ))
        .id();
    commands.spawn((
        ChildOf(camera_zoom),
        <BindingOf as Relationship>::from(camera_zoom),
        Binding::mouse_wheel(),
    ));
}
