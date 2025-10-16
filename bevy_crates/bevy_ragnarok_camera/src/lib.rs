//! Ragnarok Camera

pub mod plugin;

use std::ops::Range;

#[cfg(feature = "reflect")]
use bevy_ecs::reflect::ReflectComponent;
use bevy_ecs::{component::Component, entity::Entity, event::EntityEvent};
#[cfg(feature = "reflect")]
use bevy_reflect::Reflect;

/// Links to the entity with [`Camera`](bevy_camera::Camera).
#[derive(Component)]
#[relationship_target(relationship=CameraOfOrbitalCamera)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct OrbitalCamera(Entity);

/// This [`Camera`](bevy_camera::Camera) is part of the
/// linked [`OrbitalCamera`].
#[derive(Component)]
#[relationship(relationship_target=OrbitalCamera)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct CameraOfOrbitalCamera(Entity);

/// Pitch, Yaw, and Zoom of the [`OrbitalCamera`]
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct OrbitalCameraSettings {
    /// The top-down angle of the camera, this will have a very limited
    /// range of motion
    pub pitch: f32,
    /// The left-right rotation of the camera, this can rotate 360 degrees
    pub yaw: f32,
    /// How close the camera will be from the character
    pub zoom: f32,
}

/// Limits for [`OrbitalCameraSettings::pitch`] and [`OrbitalCameraSettings::zoom`].
#[derive(Component)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
#[component(immutable)]
pub struct OrbitalCameraLimits {
    /// The default value for [`OrbitalCameraSettings::yaw`]
    pub yaw_default: f32,
    /// The range that [`OrbitalCameraSettings::yaw`] can take
    pub yaw_range: Range<f32>,
    /// The default value for [`OrbitalCameraSettings::pitch`]
    pub pitch_default: f32,
    /// The range that [`OrbitalCameraSettings::pitch`] can take
    pub pitch_range: Range<f32>,
    /// The default value for [`OrbitalCameraSettings::zoom`]
    pub zoom_default: f32,
    /// The range that [`OrbitalCameraSettings::zoom`] can take
    pub zoom_range: Range<f32>,
}

/// This [`Entity`] is being tracked by a [`OrbitalCamera`]
#[derive(Component)]
#[relationship(relationship_target=TrackingEntity)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct TrackedEntity(Entity);

/// This [`OrbitalCamera`] is tracking and entity
#[derive(Component)]
#[relationship_target(relationship=TrackedEntity)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Component))]
pub struct TrackingEntity(Entity);

/// Marker component that indicates that the [`OrbitalCamera`]
/// is resetting its pitch
#[derive(Component)]
pub struct ResettingCameraPitch;

/// Event to reset [`OrbitalCamera`]'s pitch
#[derive(EntityEvent)]
pub struct ResetCameraPitch(Entity);

impl From<Entity> for ResetCameraPitch {
    fn from(value: Entity) -> Self {
        Self(value)
    }
}
