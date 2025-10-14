use bevy::math::Vec2;
use bevy_enhanced_input::prelude::InputAction;

pub mod plugin;

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
