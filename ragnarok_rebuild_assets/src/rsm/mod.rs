mod error;
pub mod mesh;
mod volume_box;

use std::io::{self, Read};

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use super::{common::Version, read_n_euc_kr_strings};

pub use self::{error::Error, volume_box::VolumeBox};

type TextureAndMeshNames = (Box<[Box<str>]>, Box<[Box<str>]>);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShadeType {
    Unlit,
    Flat,
    Smooth,
}

#[derive(Debug)]
pub struct Rsm {
    pub signature: Box<str>,
    pub version: Version,
    pub animation_length: i32,
    pub shade_type: ShadeType,
    pub alpha: u8,
    pub frames_per_second: f32,
    pub textures: Box<[Box<str>]>,
    pub root_meshes: Box<[Box<str>]>,
    pub meshes: Box<[mesh::Mesh]>,
    pub scale_key_frames: Box<[mesh::ScaleKeyFrame]>,
    pub volume_boxes: Box<[VolumeBox]>,
}

impl Rsm {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, self::Error> {
        let signature = Self::read_signature(reader)?;
        let version = Version::rsm_version_from_reader(reader)?;
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

        let frames_per_second = if version >= Version(2, 2, 0) {
            reader.read_le_f32()?
        } else {
            0.
        };

        // Skip 16 bytes
        if version < Version(2, 2, 0) {
            let _padding = reader.read_vec(16)?;
        }

        let (textures, root_meshes) = Self::read_textures_and_meshs_names(reader, &version)?;

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

        let volume_boxes = {
            match reader.read_le_u32() {
                Ok(count) => (0..count)
                    .map(|_| VolumeBox::from_reader(reader, &version))
                    .collect::<Result<Box<[VolumeBox]>, io::Error>>()?,
                Err(err) => {
                    // V2.3 files seems to have a 50/50 on whether they have volume boxes or not
                    if err.kind().eq(&io::ErrorKind::UnexpectedEof) {
                        log::debug!("RSM V{version} did not have a volume boxes section.");
                        [].into()
                    } else {
                        return Err(self::Error::Io(err));
                    }
                }
            }
        };

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
            animation_length,
            shade_type,
            alpha,
            frames_per_second,
            textures,
            root_meshes,
            meshes,
            scale_key_frames,
            volume_boxes,
        })
    }

    fn read_signature(mut reader: &mut dyn Read) -> Result<Box<str>, Error> {
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

    fn read_textures_and_meshs_names(
        mut reader: &mut dyn Read,
        version: &Version,
    ) -> Result<TextureAndMeshNames, self::Error> {
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

        let mesh_names = if version >= &Version(2, 2, 0) {
            let count = reader.read_le_u32()?;
            read_n_euc_kr_strings(reader, count, None)?
        } else {
            read_n_euc_kr_strings(reader, 1, Some(40))?
        };

        Ok((textures, mesh_names))
    }
}
