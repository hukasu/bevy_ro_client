mod face;
mod position_key_frame;
mod rotation_key_frame;
mod scale_key_frame;
mod texture_animation;
mod texture_uv;
#[cfg(feature = "warning")]
mod warnings;

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
use bevy_math::{Mat4, Quat, Vec3, curve::UnevenSampleAutoCurve};
#[cfg(feature = "bevy")]
use bevy_render::primitives::Aabb;
#[cfg(feature = "bevy")]
use bevy_transform::components::Transform;

#[cfg(feature = "warning")]
use ragnarok_rebuild_common::warning::Warnings;
use ragnarok_rebuild_common::{Version, euc_kr::read_n_euc_kr_strings, reader_ext::ReaderExt};

#[cfg(feature = "bevy")]
use crate::AnimationDuration;

#[cfg(feature = "warning")]
pub use self::warnings::Warning;
pub use self::{
    face::Face, position_key_frame::PositionKeyFrame, rotation_key_frame::RotationKeyFrame,
    scale_key_frame::ScaleKeyFrame, texture_animation::TextureAnimation, texture_uv::TextureUV,
};

#[derive(Debug)]
pub struct Mesh {
    pub name: Box<str>,
    pub parent_name: Box<str>,
    pub textures: Textures,
    pub transformation_matrix: [f32; 9],
    pub transformation: Transformation,
    pub vertices: Box<[[f32; 3]]>,
    pub uvs: Box<[TextureUV]>,
    pub faces: Box<[Face]>,
    pub scale_key_frames: Box<[ScaleKeyFrame]>,
    pub rotation_key_frames: Box<[RotationKeyFrame]>,
    pub position_key_frames: Box<[PositionKeyFrame]>,
    pub texture_animations: Box<[TextureAnimation]>,
}

impl Mesh {
    pub fn from_reader<R: Read>(
        reader: &mut R,
        version: &Version,
        #[cfg(feature = "warning")] texture_count: usize,
        #[cfg(feature = "warning")] warnings: &mut Warnings<super::Warning>,
    ) -> Result<Self, super::Error> {
        let (name, parent_name) = Self::read_name(reader, version)?;

        #[cfg(feature = "warning")]
        let mut mesh_warnings = Warnings::default();

        let textures = Self::read_textures_and_texture_indexes(reader, version)?;
        #[cfg(feature = "warning")]
        textures.warn(texture_count, &mut mesh_warnings);

        let transformation_matrix = Self::read_transformation_matrix(reader)?;

        let transformation = Self::read_transformation(reader, version)?;

        let vertices = Self::read_vertices(reader)?;

        let uvs = Self::read_uvs(reader, version)?;

        let faces = Self::read_faces(reader, version)?;

        let scale_key_frames = Self::read_scale_key_frames(reader, version)?;

        let rotation_key_frames = Self::read_rotation_key_frames(reader)?;

        let position_key_frames = Self::read_position_key_frames(reader, version)?;

        let texture_animations = Self::read_texture_key_frames(reader, version)?;

        #[cfg(feature = "warning")]
        if !mesh_warnings.is_empty() {
            warnings.push(super::Warning::MeshWarnings(name.clone(), mesh_warnings));
        }

        Ok(Self {
            name,
            parent_name,
            textures,
            transformation_matrix,
            transformation,
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
    ) -> Result<Textures, super::Error> {
        if version >= &Version(2, 3, 0) {
            let count = reader.read_le_u32()?;
            let textures = read_n_euc_kr_strings(reader, count, None)?;

            Ok(Textures::Paths(textures))
        } else {
            let texture_indexes = {
                let count = reader.read_le_u32()?;
                (0..count)
                    .map(|_| reader.read_le_i32())
                    .collect::<Result<Box<[i32]>, io::Error>>()?
            };
            Ok(Textures::Indexes(texture_indexes))
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

    fn read_transformation<R: Read>(
        reader: &mut R,
        version: &Version,
    ) -> Result<Transformation, super::Error> {
        if version >= &Version(2, 2, 0) {
            let position = [
                reader.read_le_f32()?,
                reader.read_le_f32()?,
                reader.read_le_f32()?,
            ];
            Ok(Transformation::Simple(position))
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
            Ok(Transformation::Complete {
                offset,
                position,
                rotation_angle,
                rotation_axis,
                scale,
            })
        }
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

        let mut mesh_attributes = MeshAttributes::default();

        let mut attributes = HashMap::new();

        for face in &self.faces {
            let Some(texture_index) = self.textures.index(usize::from(face.texture_id)) else {
                #[cfg(feature = "bevy")]
                bevy_log::warn!(
                    "Mesh face had textures that are not addressable on current architecture."
                );
                continue;
            };
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
                            .entry((texture_index, face.two_side == 1))
                            .and_modify(|indexes| indexes.push(cur_len))
                            .or_insert(vec![cur_len]);
                    }
                    Entry::Occupied(o) => {
                        let Some(entry) = mesh_attributes
                            .indexes
                            .get_mut(&(texture_index, face.two_side == 1))
                        else {
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
                .transform_point(transformation_matrix.transform_point3(Vec3::from_array(*vertex)))
        }))
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn transformation_matrix(&self) -> Mat4 {
        let offset = self.transformation.offset();
        Mat4 {
            x_axis: Vec3::from_slice(&self.transformation_matrix[0..3]).extend(0.),
            y_axis: Vec3::from_slice(&self.transformation_matrix[3..6]).extend(0.),
            z_axis: Vec3::from_slice(&self.transformation_matrix[6..9]).extend(0.),
            w_axis: offset.extend(1.),
        }
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn transform(&self) -> Transform {
        self.transformation.transform()
    }

    #[cfg(feature = "bevy")]
    #[must_use]
    pub fn recentered_transform(&self, mesh_bounds: &Aabb) -> Transform {
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

#[derive(Debug)]
pub enum Textures {
    Paths(Box<[Box<str>]>),
    Indexes(Box<[i32]>),
}

impl Textures {
    fn index(&self, index: usize) -> Option<i32> {
        match self {
            Self::Paths(_) => i32::try_from(index).ok(),
            Self::Indexes(indexes) => indexes.get(index).copied(),
        }
    }

    #[cfg(feature = "warning")]
    fn warn(
        &self,
        #[cfg(feature = "warning")] texture_count: usize,
        #[cfg(feature = "warning")] warnings: &mut Warnings<warnings::Warning>,
    ) {
        if let Self::Indexes(indexes) = &self {
            for index in indexes {
                if *index < 0 {
                    warnings.push(warnings::Warning::TextureOutOfBounds(texture_count, *index));
                }
                if let Ok(index_usize) = usize::try_from(*index) {
                    if index_usize >= texture_count {
                        warnings.push(warnings::Warning::TextureOutOfBounds(texture_count, *index));
                    }
                } else {
                    warnings.push(warnings::Warning::CantBeAddressed(*index));
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Transformation {
    Complete {
        offset: [f32; 3],
        position: [f32; 3],
        rotation_angle: f32,
        rotation_axis: [f32; 3],
        scale: [f32; 3],
    },
    Simple([f32; 3]),
}

impl Transformation {
    #[cfg(feature = "bevy")]
    pub fn offset(&self) -> Vec3 {
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

    #[cfg(feature = "bevy")]
    pub fn transform(&self) -> Transform {
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
