use bevy::{ecs::component::Component, math::Vec2};
use bevy_enhanced_input::prelude::InputAction;

pub mod plugin;

/// The primary context for camera controls.
#[derive(Component)]
pub struct OrbitalCameraPrimaryContext;

/// The secondary context for camera controls.
#[derive(Component)]
pub struct OrbitalCameraSecondaryContext;

/// Action to change camera yaw
#[derive(InputAction)]
#[action_output(Vec2)]
pub struct CameraYaw;

/// Action to change camera pitch
#[derive(InputAction)]
#[action_output(Vec2)]
pub struct CameraPitch;

/// Action to change camera zoom
#[derive(InputAction)]
#[action_output(Vec2)]
pub struct CameraZoom;

/// Action to reset camera yaw to default
#[derive(InputAction)]
#[action_output(bool)]
pub struct ResetCameraYaw;

/// Action to reset camera pitch to default
#[derive(InputAction)]
#[action_output(bool)]
pub struct ResetCameraPitch;
