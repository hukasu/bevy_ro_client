#[cfg(feature = "bevy")]
pub mod components;
mod error;
pub mod mesh;
#[cfg(feature = "bevy")]
pub mod plugin;
mod volume_box;
#[cfg(feature = "warning")]
pub mod warnings;

use std::{
    collections::HashSet,
    io::{self, Read},
};

#[cfg(feature = "bevy")]
use bevy_animation::{
    animated_field,
    prelude::{AnimatableCurve, AnimatedField, AnimationCurve},
};
#[cfg(feature = "bevy")]
use bevy_asset::Asset;
#[cfg(feature = "bevy")]
use bevy_math::{Vec3, curve::UnevenSampleAutoCurve};
#[cfg(feature = "bevy")]
use bevy_reflect::TypePath;
#[cfg(feature = "bevy")]
use bevy_transform::components::Transform;

#[cfg(feature = "warning")]
use ragnarok_rebuild_common::warning::Warnings;
use ragnarok_rebuild_common::{Version, euc_kr::read_n_euc_kr_strings, reader_ext::ReaderExt};

#[cfg(feature = "warning")]
use self::warnings::Warning;
pub use self::{error::Error, volume_box::VolumeBox};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadeType {
    Unlit,
    Flat,
    Smooth,
}

#[derive(Debug)]
#[cfg_attr(feature = "bevy", derive(Asset, TypePath))]
pub struct Rsm {
    pub signature: Box<str>,
    pub version: Version,
    pub animation_duration: AnimationDuration,
    pub shade_type: ShadeType,
    pub alpha: u8,
    pub textures: Box<[Box<str>]>,
    pub root_meshes: Box<[Box<str>]>,
    pub meshes: Box<[mesh::Mesh]>,
    pub scale_key_frames: Box<[mesh::ScaleKeyFrame]>,
    pub volume_boxes: Box<[VolumeBox]>,
    #[cfg(feature = "warning")]
    pub warnings: Warnings<Warning>,
}

impl Rsm {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self, self::Error> {
        #[cfg(feature = "warning")]
        let mut warnings = Warnings::default();

        let signature = Self::read_signature(reader)?;
        let version = Self::read_version(reader)?;
        let animation_length = reader.read_le_i32()?;
        let shade_type = match reader.read_le_i32()? {
            0 => ShadeType::Unlit,
            1 => ShadeType::Flat,
            2 => ShadeType::Smooth,
            invalid => {
                return Err(self::Error::InvalidShadeType(invalid));
            }
        };

        let alpha = if version >= Version(1, 4, 0) {
            reader.read_u8()?
        } else {
            0xff
        };

        let animation_duration = if version >= Version(2, 2, 0) {
            let per_second = reader.read_le_f32()?;
            AnimationDuration::PerSecond(animation_length as f32, per_second)
        } else {
            AnimationDuration::Simple(animation_length as f32)
        };

        // Skip 16 bytes
        if version < Version(2, 2, 0) {
            let _padding = reader.read_vec(16)?;
        }

        let textures = Self::read_textures(reader, &version)?;
        let root_meshes = Self::read_meshs_names(
            reader,
            &version,
            #[cfg(feature = "warning")]
            &mut warnings,
        )?;

        let meshes = {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| mesh::Mesh::from_reader(reader, &version))
                .collect::<Result<Box<[mesh::Mesh]>, self::Error>>()?
        };

        let scale_key_frames = if version < Version(1, 6, 0) {
            let count = reader.read_le_u32()?;
            (0..count)
                .map(|_| mesh::ScaleKeyFrame::from_reader(reader))
                .collect::<Result<Box<[mesh::ScaleKeyFrame]>, io::Error>>()?
        } else {
            [].into()
        };

        let volume_boxes = Self::read_volume_boxes(
            reader,
            &version,
            #[cfg(feature = "warning")]
            &mut warnings,
        )?;

        if version >= Version(1, 5, 0) && version < Version(1, 6, 0) {
            // All V1.5 seems to have this 4 bytes at the end of file
            let _padding = reader.read_le_u32()?;
        }

        let mut rest = vec![];
        reader.read_to_end(&mut rest)?;
        if !rest.is_empty() {
            return Err(Error::IncompleteRead(version, rest.len()));
        }

        Ok(Self {
            signature,
            version,
            animation_duration,
            shade_type,
            alpha,
            textures,
            root_meshes,
            meshes,
            scale_key_frames,
            volume_boxes,
            #[cfg(feature = "warning")]
            warnings,
        })
    }

    fn read_signature<R: Read>(reader: &mut R) -> Result<Box<str>, Error> {
        let signature = {
            let buffer: [u8; 4] = reader.read_array()?;
            String::from_utf8(buffer.to_vec())
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Read invalid Utf8.")
                })?
                .into_boxed_str()
        };
        if (*signature).eq("GRSM") {
            Ok(signature)
        } else {
            Err(Error::InvalidSignature(signature))
        }
    }

    fn read_version<R: Read>(reader: &mut R) -> Result<Version, error::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        Ok(Version(major, minor, 0))
    }

    fn read_textures(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Box<[Box<str>]>, self::Error> {
        let textures = if version < &Version(2, 3, 0) {
            let len = if version >= &Version(2, 2, 0) {
                None
            } else {
                Some(40)
            };

            let count = reader.read_le_u32()?;
            read_n_euc_kr_strings(reader, count, len)?
        } else {
            [].into()
        };

        Ok(textures)
    }

    fn read_meshs_names(
        mut reader: &mut dyn Read,
        version: &Version,
        #[cfg(feature = "warning")] warnings: &mut Warnings<Warning>,
    ) -> Result<Box<[Box<str>]>, self::Error> {
        let mesh_names = if version >= &Version(2, 2, 0) {
            let count = reader.read_le_u32()?;
            read_n_euc_kr_strings(reader, count, None)?
        } else {
            read_n_euc_kr_strings(reader, 1, Some(40))?
        };

        #[cfg(feature = "warning")]
        if mesh_names.is_empty() {
            warnings.push(Warning::EmptyRootMeshes);
        }

        #[cfg(feature = "warning")]
        {
            let mut set = HashSet::new();
            let mut set2 = HashSet::new();
            for mesh_name in &mesh_names {
                if mesh_name.is_empty() {
                    warnings.push(Warning::BlankRootMeshName);
                }
                if !set.insert(mesh_name) && set2.insert(mesh_name) {
                    warnings.push(Warning::DuplicateRootMeshName(mesh_name.clone()));
                }
            }
        }

        Ok(mesh_names)
    }

    fn read_volume_boxes<R: Read>(
        reader: &mut R,
        version: &Version,
        #[cfg(feature = "warning")] warnings: &mut Warnings<Warning>,
    ) -> Result<Box<[VolumeBox]>, error::Error> {
        let volume_boxes = match reader.read_le_u32() {
            Ok(count) => (0..count)
                .map(|_| VolumeBox::from_reader(reader, version))
                .collect::<Result<Box<[VolumeBox]>, io::Error>>()?,
            Err(err) => {
                // V2.3 files seems to have a 50/50 on whether they have volume boxes or not
                if err.kind().eq(&io::ErrorKind::UnexpectedEof) {
                    #[cfg(feature = "warning")]
                    warnings.push(Warning::MissingVolumeBoxSection);
                    #[cfg(not(feature = "warning"))]
                    bevy_log::warn!("RSM V{version} did not have a volume boxes section.");

                    [].into()
                } else {
                    return Err(self::Error::Io(err));
                }
            }
        };

        #[cfg(feature = "warning")]
        if !volume_boxes.is_empty() {
            warnings.push(Warning::NonEmptyVolumeBox);
        }

        Ok(volume_boxes)
    }

    #[cfg(feature = "bevy")]
    fn scale_animation_curve(&self) -> Option<impl AnimationCurve> {
        if !self.scale_key_frames.is_empty() {
            match UnevenSampleAutoCurve::new(
                self.scale_key_frames
                    .iter()
                    .map(|frame| self.animation_duration.transform(frame.frame as f32))
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
                    bevy_log::error!("Failed to build scale animation due to `{err}`.");
                    None
                }
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AnimationDuration {
    /// Duration is given as a multiple of 1000
    Simple(f32),
    /// Duration is given as a number of frames and number of frames per second
    PerSecond(f32, f32),
}

impl AnimationDuration {
    pub fn duration(&self) -> f32 {
        match self {
            Self::Simple(duration) => duration / 1000.,
            Self::PerSecond(duration, per) => duration / per,
        }
    }

    pub fn transform(&self, frame: f32) -> f32 {
        match self {
            Self::Simple(_) => frame / 1000.,
            Self::PerSecond(_, per) => frame / per,
        }
    }
}
