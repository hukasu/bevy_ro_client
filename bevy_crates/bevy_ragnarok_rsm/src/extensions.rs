//! Extensions to Ragnarok Online's Rsm files

use bevy_animation::{
    animated_field,
    animation_curves::{AnimatableCurve, AnimatedField, AnimationCurve},
};
use bevy_asset::RenderAssetUsages;
use bevy_camera::primitives::Aabb;
use bevy_math::{Mat4, Quat, Vec3, curve::UnevenSampleAutoCurve};
use bevy_transform::components::Transform;
use ragnarok_rsm::{
    AnimationDuration, Rsm,
    mesh::{Mesh, Primitive, Transformation},
};

/// Extension trait for the root [`Rsm`] file
pub trait RsmExt {
    #[must_use]
    fn position_animation_curve(&self) -> Option<impl AnimationCurve>;
}

impl RsmExt for Rsm {
    fn position_animation_curve(&self) -> Option<impl AnimationCurve> {
        if !self.position_key_frames.is_empty() {
            let root_mesh = self
                .meshes
                .iter()
                .find(|mesh| mesh.name == self.root_meshes[0])?;
            let root_mesh_bounds = root_mesh.bounds()?;
            let correction = Vec3::new(
                root_mesh_bounds.center.x,
                root_mesh_bounds.max().y,
                root_mesh_bounds.center.z,
            );

            match UnevenSampleAutoCurve::new(
                self.position_key_frames
                    .iter()
                    .map(|frame| self.animation_duration.transform(frame.frame as f32))
                    .zip(
                        self.position_key_frames
                            .iter()
                            .map(|frame| Vec3::from_array(frame.position) - correction),
                    ),
            ) {
                Ok(uneven_curve) => {
                    let animatable_curve =
                        AnimatableCurve::new(animated_field!(Transform::translation), uneven_curve);
                    Some(animatable_curve)
                }
                Err(err) => {
                    log::error!("Failed to build position animation due to `{err}`.");
                    None
                }
            }
        } else {
            None
        }
    }
}

/// Extension for a [`Rsm`]'s [`Mesh`].
pub trait RsmMeshExt {
    #[must_use]
    fn bounds(&self) -> Option<Aabb>;

    #[must_use]
    fn transformation_matrix(&self) -> Mat4;

    #[must_use]
    fn transform(&self) -> Transform;

    #[must_use]
    fn recentered_transform(&self, mesh_bounds: &Aabb) -> Transform;

    #[must_use]
    fn position_animation_curve(
        &self,
        animation_duration: AnimationDuration,
    ) -> Option<impl AnimationCurve>;

    #[must_use]
    fn rotation_animation_curve(
        &self,
        animation_duration: AnimationDuration,
    ) -> Option<impl AnimationCurve>;

    #[must_use]
    fn scale_animation_curve(
        &self,
        animation_duration: AnimationDuration,
    ) -> Option<impl AnimationCurve>;
}

impl RsmMeshExt for Mesh {
    fn bounds(&self) -> Option<Aabb> {
        let transformation_matrix = self.transformation_matrix();
        let transform = self.transform();

        Aabb::enclosing(self.vertices.iter().map(move |vertex| {
            transform
                .transform_point(transformation_matrix.transform_point3(Vec3::from_array(*vertex)))
        }))
    }

    fn transformation_matrix(&self) -> Mat4 {
        let offset = self.transformation.offset();
        Mat4 {
            x_axis: Vec3::from_slice(&self.transformation_matrix[0..3]).extend(0.),
            y_axis: Vec3::from_slice(&self.transformation_matrix[3..6]).extend(0.),
            z_axis: Vec3::from_slice(&self.transformation_matrix[6..9]).extend(0.),
            w_axis: offset.extend(1.),
        }
    }

    fn transform(&self) -> Transform {
        self.transformation.transform()
    }

    fn recentered_transform(&self, mesh_bounds: &Aabb) -> Transform {
        let mut transform = self.transformation.transform();
        if !matches!(self.transformation, Transformation::Simple(_)) {
            transform.translation -= Vec3::new(
                mesh_bounds.center.x,
                mesh_bounds.max().y,
                mesh_bounds.center.z,
            );
        }
        transform
    }

    fn position_animation_curve(
        &self,
        animation_duration: AnimationDuration,
    ) -> Option<impl AnimationCurve> {
        if !self.position_key_frames.is_empty() {
            match UnevenSampleAutoCurve::new(
                self.position_key_frames
                    .iter()
                    .map(|frame| animation_duration.transform(frame.frame as f32))
                    .zip(
                        self.position_key_frames
                            .iter()
                            .map(|frame| Vec3::from_array(frame.position)),
                    ),
            ) {
                Ok(uneven_curve) => {
                    let animatable_curve =
                        AnimatableCurve::new(animated_field!(Transform::translation), uneven_curve);
                    Some(animatable_curve)
                }
                Err(err) => {
                    log::error!(
                        "Failed to build position animation of {} due to `{err}`.",
                        self.name
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    fn rotation_animation_curve(
        &self,
        animation_duration: AnimationDuration,
    ) -> Option<impl AnimationCurve> {
        if !self.rotation_key_frames.is_empty() {
            match UnevenSampleAutoCurve::new(
                self.rotation_key_frames
                    .iter()
                    .map(|frame| animation_duration.transform(frame.frame as f32))
                    .zip(
                        self.rotation_key_frames
                            .iter()
                            .map(|frame| Quat::from_array(frame.quaternion)),
                    ),
            ) {
                Ok(uneven_curve) => {
                    let animatable_curve =
                        AnimatableCurve::new(animated_field!(Transform::rotation), uneven_curve);
                    Some(animatable_curve)
                }
                Err(err) => {
                    log::error!(
                        "Failed to build rotation animation of {} due to `{err}`.",
                        self.name
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    fn scale_animation_curve(
        &self,
        animation_duration: AnimationDuration,
    ) -> Option<impl AnimationCurve> {
        if !self.scale_key_frames.is_empty() {
            match UnevenSampleAutoCurve::new(
                self.scale_key_frames
                    .iter()
                    .map(|frame| animation_duration.transform(frame.frame as f32))
                    .zip(
                        self.scale_key_frames
                            .iter()
                            .map(|frame| Vec3::from_array(frame.scale)),
                    ),
            ) {
                Ok(uneven_curve) => {
                    let animatable_curve =
                        AnimatableCurve::new(animated_field!(Transform::scale), uneven_curve);
                    Some(animatable_curve)
                }
                Err(err) => {
                    log::error!(
                        "Failed to build scale animation of {} due to `{err}`.",
                        self.name
                    );
                    None
                }
            }
        } else {
            None
        }
    }
}

/// Extension for a [`Rsm`] [`Mesh`]'s [`Primitive`].
pub trait RsmPrimitiveExt {
    fn into_mesh(self) -> bevy_mesh::Mesh;
}

impl RsmPrimitiveExt for Primitive {
    fn into_mesh(self) -> bevy_mesh::Mesh {
        bevy_mesh::Mesh::new(
            bevy_mesh::PrimitiveTopology::TriangleList,
            if cfg!(feature = "debug") {
                RenderAssetUsages::all()
            } else {
                RenderAssetUsages::RENDER_WORLD
            },
        )
        .with_inserted_attribute(bevy_mesh::Mesh::ATTRIBUTE_POSITION, self.vertices)
        .with_inserted_attribute(bevy_mesh::Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(bevy_mesh::Mesh::ATTRIBUTE_UV_0, self.uv)
        .with_inserted_attribute(bevy_mesh::Mesh::ATTRIBUTE_COLOR, self.color)
        .with_inserted_indices(bevy_mesh::Indices::U16(self.indices))
    }
}

/// Extension for a [`Rsm`] [`Mesh`]'s [`Transformation`].
pub trait RsmTransformationExt {
    fn offset(&self) -> Vec3;

    fn transform(&self) -> Transform;
}

impl RsmTransformationExt for Transformation {
    fn offset(&self) -> Vec3 {
        match self {
            Transformation::Complete {
                offset,
                position: _,
                rotation_angle: _,
                rotation_axis: _,
                scale: _,
            } => Vec3::from_array(*offset),
            Transformation::Simple(_) => Vec3::ZERO,
        }
    }

    fn transform(&self) -> Transform {
        match self {
            Self::Complete {
                offset: _,
                position,
                rotation_angle,
                rotation_axis,
                scale,
            } => {
                let translation = Vec3::from_array(*position);
                let rotation = {
                    let rotation_axis = Vec3::from_array(*rotation_axis);
                    if rotation_axis.length() <= 0. {
                        Quat::default()
                    } else {
                        Quat::from_axis_angle(rotation_axis, *rotation_angle)
                    }
                };
                let scale = Vec3::from_array(*scale);

                Transform {
                    translation,
                    rotation,
                    scale,
                }
            }
            Self::Simple(position) => Transform::from_translation(Vec3::from_array(*position)),
        }
    }
}
