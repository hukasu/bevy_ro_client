mod face;
mod position_key_frame;
mod rotation_key_frame;
mod scale_key_frame;
mod texture_animation;
mod texture_uv;

use std::{
    collections::{HashMap, hash_map::Entry},
    io::{self, Read},
};

#[cfg(feature = "bevy")]
use bevy_animation::{
    animated_field,
    prelude::{AnimatableCurve, AnimatedField, AnimationCurve},
};
#[cfg(feature = "bevy")]
use bevy_math::{Mat3, Mat4, Quat, Vec3, curve::UnevenSampleAutoCurve};
#[cfg(feature = "bevy")]
use bevy_render::primitives::Aabb;
#[cfg(feature = "bevy")]
use bevy_transform::components::Transform;

use ragnarok_rebuild_common::{Version, euc_kr::read_n_euc_kr_strings, reader_ext::ReaderExt};

pub use self::{
    face::Face, position_key_frame::PositionKeyFrame, rotation_key_frame::RotationKeyFrame,
    scale_key_frame::ScaleKeyFrame, texture_animation::TextureAnimation, texture_uv::TextureUV,
};
#[cfg(feature = "bevy")]
use crate::AnimationDuration;

type TexturePathsAndIndexes = (Box<[Box<str>]>, Box<[i32]>);

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
    pub fn from_reader<R: Read>(reader: &mut R, version: &Version) -> Result<Self, super::Error> {
        let (name, parent_name) = Self::read_name(reader, version)?;

        let (textures, texture_indexes) = Self::read_textures_and_texture_indexes(reader, version)?;

        let transformation_matrix = Self::read_transformation_matrix(reader)?;

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

        let vertices = Self::read_vertices(reader)?;

        let uvs = Self::read_uvs(reader, version)?;

        let faces = Self::read_faces(reader, version)?;

        let scale_key_frames = Self::read_scale_key_frames(reader, version)?;

        let rotation_key_frames = Self::read_rotation_key_frames(reader)?;

        let position_key_frames = Self::read_position_key_frames(reader, version)?;

        let texture_animations = Self::read_texture_key_frames(reader, version)?;

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

    fn read_name<R: Read>(
        reader: &mut R,
        version: &Version,
    ) -> Result<(Box<str>, Box<str>), super::Error> {
        if version >= &Version(2, 2, 0) {
            let [ref name, ref parent_name] = *read_n_euc_kr_strings(reader, 2, None)? else {
                return Err(super::Error::InvalidMeshName);
            };
            Ok((name.clone(), parent_name.clone()))
        } else {
            let [ref name, ref parent_name] = *read_n_euc_kr_strings(reader, 2, Some(40))? else {
                return Err(super::Error::InvalidMeshName);
            };
            Ok((name.clone(), parent_name.clone()))
        }
    }

    fn read_textures_and_texture_indexes<R: Read>(
        reader: &mut R,
        version: &Version,
    ) -> Result<TexturePathsAndIndexes, super::Error> {
        if version >= &Version(2, 3, 0) {
            let count = reader.read_le_u32()?;
            let textures = read_n_euc_kr_strings(reader, count, None)?;

            Ok((textures, (0..count as i32).collect()))
        } else {
            let texture_indexes = {
                let count = reader.read_le_u32()?;
                (0..count)
                    .map(|_| reader.read_le_i32())
                    .collect::<Result<Box<[i32]>, io::Error>>()?
            };
            Ok(([].into(), texture_indexes))
        }
    }

    fn read_transformation_matrix<R: Read>(reader: &mut R) -> Result<[f32; 9], super::Error> {
        Ok([
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
            reader.read_le_f32()?,
        ])
    }

    fn read_vertices<R: Read>(reader: &mut R) -> Result<Box<[[f32; 3]]>, io::Error> {
        let count = reader.read_le_u32()?;
        (0..count)
            .map(|_| -> Result<[f32; 3], io::Error> {
                Ok([
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ])
            })
            .collect::<Result<Box<[[f32; 3]]>, io::Error>>()
    }

    fn read_uvs<R: Read>(reader: &mut R, version: &Version) -> Result<Box<[TextureUV]>, io::Error> {
        let count = reader.read_le_u32()?;
        (0..count)
            .map(|_| TextureUV::from_reader(reader, version))
            .collect::<Result<Box<[TextureUV]>, io::Error>>()
    }

    fn read_faces<R: Read>(reader: &mut R, version: &Version) -> Result<Box<[Face]>, io::Error> {
        let count = reader.read_le_u32()?;
        (0..count)
            .map(|_| Face::from_reader(reader, version))
            .collect::<Result<Box<[Face]>, io::Error>>()
    }

    fn read_scale_key_frames<R: Read>(
        reader: &mut R,
        version: &Version,
    ) -> Result<Box<[ScaleKeyFrame]>, io::Error> {
        if version >= &Version(1, 6, 0) {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| ScaleKeyFrame::from_reader(reader))
                .collect::<Result<Box<[ScaleKeyFrame]>, io::Error>>()
        } else {
            Ok([].into())
        }
    }

    fn read_rotation_key_frames<R: Read>(
        reader: &mut R,
    ) -> Result<Box<[RotationKeyFrame]>, io::Error> {
        let count = reader.read_le_u32()?;
        (0..count)
            .map(|_| RotationKeyFrame::from_reader(reader))
            .collect::<Result<Box<[RotationKeyFrame]>, io::Error>>()
    }

    fn read_position_key_frames<R: Read>(
        reader: &mut R,
        version: &Version,
    ) -> Result<Box<[PositionKeyFrame]>, io::Error> {
        if version >= &Version(2, 2, 0) {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| PositionKeyFrame::from_reader(reader))
                .collect::<Result<Box<[PositionKeyFrame]>, io::Error>>()
        } else {
            Ok([].into())
        }
    }

    fn read_texture_key_frames<R: Read>(
        reader: &mut R,
        version: &Version,
    ) -> Result<Box<[TextureAnimation]>, io::Error> {
        if version >= &Version(2, 3, 0) {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| TextureAnimation::from_reader(reader))
                .collect::<Result<Box<[TextureAnimation]>, io::Error>>()
        } else {
            Ok([].into())
        }
    }

    pub fn flat_mesh(&self) -> MeshAttributes {
        let vertices = &self.vertices;
        let uvs = &self.uvs;
        let texture_index = &self.texture_indexes;

        let mut mesh_attributes = MeshAttributes::default();

        let mut attributes = HashMap::new();

        for face in &self.faces {
            for (vertex, uv) in face.vertices.iter().copied().zip(face.uv) {
                #[expect(
                    clippy::unwrap_used,
                    reason = "Should never have more than u16 vertexes"
                )]
                let cur_len = u16::try_from(attributes.len()).unwrap();
                match attributes.entry((vertex, uv, face.texture_id, face.two_side)) {
                    Entry::Vacant(v) => {
                        v.insert(cur_len);
                        mesh_attributes.vertices.push(vertices[usize::from(vertex)]);
                        mesh_attributes.uv.push(uvs[usize::from(uv)].uv);
                        mesh_attributes.color.push(
                            uvs[usize::from(uv)]
                                .color
                                .map(|channel| channel as f32 / 255.),
                        );
                        mesh_attributes
                            .indexes
                            .entry((
                                texture_index[usize::from(face.texture_id)],
                                face.two_side == 1,
                            ))
                            .and_modify(|indexes| indexes.push(cur_len))
                            .or_insert(vec![cur_len]);
                    }
                    Entry::Occupied(o) => {
                        let Some(entry) = mesh_attributes.indexes.get_mut(&(
                            texture_index[usize::from(face.texture_id)],
                            face.two_side == 1,
                        )) else {
                            unreachable!("If it is occupied then an entry must exist.");
                        };
                        entry.push(*o.get());
                    }
                }
            }
        }

        mesh_attributes
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
    pub fn transformation_matrix(&self) -> Transform {
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

#[derive(Debug, Default)]
pub struct MeshAttributes {
    pub vertices: Vec<[f32; 3]>,
    pub uv: Vec<[f32; 2]>,
    pub color: Vec<[f32; 4]>,
    pub indexes: HashMap<(i32, bool), Vec<u16>>,
}
