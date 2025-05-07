mod face;
mod position_key_frame;
mod rotation_key_frame;
mod scale_key_frame;
mod texture_animation;
mod texture_uv;

use std::io::{self, Read};

#[cfg(feature = "bevy")]
use bevy_animation::{
    animated_field,
    prelude::{AnimatableCurve, AnimatedField, AnimationCurve},
};
#[cfg(feature = "bevy")]
use bevy_math::{curve::UnevenSampleAutoCurve, Mat3, Mat4, Quat, Vec3};
#[cfg(feature = "bevy")]
use bevy_render::primitives::Aabb;
#[cfg(feature = "bevy")]
use bevy_transform::components::Transform;

use ragnarok_rebuild_common::{euc_kr::read_n_euc_kr_strings, reader_ext::ReaderExt, Version};

pub use self::{
    face::Face, position_key_frame::PositionKeyFrame, rotation_key_frame::RotationKeyFrame,
    scale_key_frame::ScaleKeyFrame, texture_animation::TextureAnimation, texture_uv::TextureUV,
};
#[cfg(feature = "bevy")]
use crate::AnimationDuration;

#[derive(Debug)]
pub struct Mesh {
    pub name: Box<str>,
    pub parent_name: Box<str>,
    pub textures: Box<[Box<str>]>,
    pub texture_indexes: Box<[i32]>,
    pub transformation_matrix: [f32; 9],
    pub offset: [f32; 3],
    pub position: [f32; 3],
    pub rotation_angle: f32,
    pub rotation_axis: [f32; 3],
    pub scale: [f32; 3],
    pub vertices: Box<[[f32; 3]]>,
    pub uvs: Box<[TextureUV]>,
    pub faces: Box<[Face]>,
    pub scale_key_frames: Box<[ScaleKeyFrame]>,
    pub rotation_key_frames: Box<[RotationKeyFrame]>,
    pub position_key_frames: Box<[PositionKeyFrame]>,
    pub texture_animations: Box<[TextureAnimation]>,
}

impl Mesh {
    pub fn from_reader(mut reader: &mut dyn Read, version: &Version) -> Result<Self, super::Error> {
        let (name, parent_name) = if version >= &Version(2, 2, 0) {
            let [ref name, ref parent_name] = *read_n_euc_kr_strings(reader, 2, None)? else {
                return Err(super::Error::InvalidMeshName);
            };
            (name.clone(), parent_name.clone())
        } else {
            let [ref name, ref parent_name] = *read_n_euc_kr_strings(reader, 2, Some(40))? else {
                return Err(super::Error::InvalidMeshName);
            };
            (name.clone(), parent_name.clone())
        };

        let (textures, texture_indexes) = if version >= &Version(2, 3, 0) {
            let count = reader.read_le_u32()?;
            let textures = read_n_euc_kr_strings(reader, count, None)?;

            (textures, (0..count as i32).collect())
        } else {
            let texture_indexes = {
                let count = reader.read_le_u32()?;
                (0..count)
                    .map(|_| reader.read_le_i32())
                    .collect::<Result<Box<[i32]>, io::Error>>()?
            };
            ([].into(), texture_indexes)
        };

        let transformation_matrix = [
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ];

        let (offset, position, rotation_angle, rotation_axis, scale) =
            if version >= &Version(2, 2, 0) {
                let offset = [0.; 3];
                let position = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                let rotation_angle = 0.;
                let rotation_axis = [0.; 3];
                let scale = [1.; 3];
                (offset, position, rotation_angle, rotation_axis, scale)
            } else {
                let offset = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                let position = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                let rotation_angle = reader.read_le_f32()?;
                let rotation_axis = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                let scale = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                (offset, position, rotation_angle, rotation_axis, scale)
            };

        let vertices = {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| -> Result<[f32; 3], io::Error> {
                    Ok([
                        reader.read_le_f32()?,
                        reader.read_le_f32()?,
                        reader.read_le_f32()?,
                    ])
                })
                .collect::<Result<Box<[[f32; 3]]>, io::Error>>()?
        };

        let uvs = {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| TextureUV::from_reader(reader, version))
                .collect::<Result<Box<[TextureUV]>, io::Error>>()?
        };

        let faces = {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| Face::from_reader(reader, version))
                .collect::<Result<Box<[Face]>, io::Error>>()?
        };

        let scale_key_frames = if version >= &Version(1, 6, 0) {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| ScaleKeyFrame::from_reader(reader))
                .collect::<Result<Box<[ScaleKeyFrame]>, io::Error>>()?
        } else {
            [].into()
        };

        let rotation_key_frames = {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| RotationKeyFrame::from_reader(reader))
                .collect::<Result<Box<[RotationKeyFrame]>, io::Error>>()?
        };

        let position_key_frames = if version >= &Version(2, 2, 0) {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| PositionKeyFrame::from_reader(reader))
                .collect::<Result<Box<[PositionKeyFrame]>, io::Error>>()?
        } else {
            [].into()
        };

        let texture_animations = if version >= &Version(2, 3, 0) {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| TextureAnimation::from_reader(reader))
                .collect::<Result<Box<[TextureAnimation]>, io::Error>>()?
        } else {
            [].into()
        };

        Ok(Self {
            name,
            parent_name,
            textures,
            texture_indexes,
            transformation_matrix,
            offset,
            position,
            rotation_angle,
            rotation_axis,
            scale,
            vertices,
            uvs,
            faces,
            scale_key_frames,
            rotation_key_frames,
            position_key_frames,
            texture_animations,
        })
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn bounds(&self) -> Option<Aabb> {
        let transformation_matrix = self.transformation_matrix();
        let transform = self.transform();

        Aabb::enclosing(self.vertices.iter().map(move |vertex| {
            transform
                .transform_point(transformation_matrix.transform_point(Vec3::from_array(*vertex)))
        }))
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    fn transformation_matrix(&self) -> Transform {
        let offset = Vec3::from_array(self.offset);
        let trasn_matrix = Mat3::from_cols_array(&self.transformation_matrix);
        Transform::from_matrix(Mat4 {
            x_axis: trasn_matrix.x_axis.extend(0.),
            y_axis: trasn_matrix.y_axis.extend(0.),
            z_axis: trasn_matrix.z_axis.extend(0.),
            w_axis: offset.extend(1.),
        })
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn transform(&self) -> Transform {
        Self::recentered_transform(self, &Aabb::from_min_max(Vec3::splat(0.), Vec3::splat(0.)))
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn recentered_transform(&self, mesh_bounds: &Aabb) -> Transform {
        let translation = Vec3::from_array(self.position)
            - Vec3::new(
                mesh_bounds.center.x,
                mesh_bounds.max().y,
                mesh_bounds.center.z,
            );
        let rotation = {
            let rotation_axis = Vec3::from_array(self.rotation_axis);
            if rotation_axis.length() <= 0. {
                Quat::default()
            } else {
                Quat::from_axis_angle(rotation_axis, self.rotation_angle)
            }
        };
        let scale = Vec3::from_array(self.scale);

        Transform {
            translation,
            rotation,
            scale,
        }
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn position_animation_curve(
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
                    bevy_log::error!(
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

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn rotation_animation_curve(
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
                    bevy_log::error!(
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

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn scale_animation_curve(
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
                    bevy_log::error!(
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
