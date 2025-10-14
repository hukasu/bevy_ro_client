//! Plugin to control the cameras

use bevy_app::PostUpdate;
use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    relationship::RelationshipTarget,
    schedule::IntoScheduleConfigs,
    system::{Populated, Query},
};
use bevy_log::error;
use bevy_math::{Dir3, Vec3};
use bevy_transform::{
    TransformSystems,
    components::{GlobalTransform, Transform},
};

#[cfg(feature = "reflect")]
use crate::TrackedEntity;
use crate::{OrbitalCamera, OrbitalCameraLimits, OrbitalCameraSettings, TrackingEntity};

/// Plugin for updating [`OrbitalCamera`].
pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(
            PostUpdate,
            (clamp_orbital_camera, track_entity)
                .chain()
                .before(TransformSystems::Propagate),
        );

        #[cfg(feature = "reflect")]
        {
            use crate::CameraOfOrbitalCamera;

            app.register_type::<OrbitalCamera>();
            app.register_type::<CameraOfOrbitalCamera>();
            app.register_type::<OrbitalCameraSettings>();
            app.register_type::<OrbitalCameraLimits>();
            app.register_type::<TrackedEntity>();
            app.register_type::<TrackingEntity>();
        }
    }
}

/// Clamps the values of [`OrbitalCameraSettings`] with [`OrbitalCameraLimits`].
fn clamp_orbital_camera(cameras: Populated<(&mut OrbitalCameraSettings, &OrbitalCameraLimits)>) {
    for (mut orbital_camera, orbital_camera_limits) in cameras.into_inner() {
        orbital_camera.pitch = orbital_camera.pitch.clamp(
            orbital_camera_limits.pitch_range.start,
            orbital_camera_limits.pitch_range.end,
        );
        orbital_camera.zoom = orbital_camera.zoom.clamp(
            orbital_camera_limits.zoom_range.start,
            orbital_camera_limits.zoom_range.end,
        );
    }
}

/// Updates the transforms of the [`OrbitalCamera`] and
/// [`CameraOfOrbitalCamera`](crate::CameraOfOrbitalCamera) based on the
/// [`OrbitalCameraSettings`].
fn track_entity(
    cameras: Populated<
        (
            Entity,
            &OrbitalCamera,
            &OrbitalCameraSettings,
            &mut Transform,
            &TrackingEntity,
        ),
        With<OrbitalCamera>,
    >,
    mut transforms: Query<&mut Transform, Without<OrbitalCamera>>,
    global_transforms: Query<&GlobalTransform>,
) {
    for (camera, orbital_camera, orbital_camera_settings, mut transform, tracking_entity) in cameras
    {
        let tracked_entity = tracking_entity.collection();
        let Ok(global_transform) = global_transforms.get(*tracked_entity) else {
            error!("{camera} is tracking an entity without GlobalTransform.");
            continue;
        };
        let actual_camera = orbital_camera.collection();
        let Ok(mut camera_transform) = transforms.get_mut(*actual_camera) else {
            unreachable!("{actual_camera} did not have Transform.");
        };

        let mut desired_transform = Transform::from_translation(global_transform.translation());
        desired_transform.rotate_axis(Dir3::X, orbital_camera_settings.pitch);
        desired_transform.rotate_axis(Dir3::Y, orbital_camera_settings.yaw);
        *transform = desired_transform;

        camera_transform.translation = Vec3::new(0., 0., orbital_camera_settings.zoom);
    }
}
