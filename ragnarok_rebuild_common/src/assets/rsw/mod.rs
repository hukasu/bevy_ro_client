mod bounding_box;
pub mod effect;
pub mod error;
pub mod light;
mod lighting_params;
pub mod model;
mod quad_tree;
pub mod sound;
pub mod version;

use std::io::Read;

use self::{
    bounding_box::BoundingBox,
    effect::Effect,
    error::Error,
    light::Light,
    lighting_params::LightingParams,
    model::Model,
    quad_tree::{QuadTree, QuadTreeRange, QUAD_TREE_SIZE},
    sound::Sound,
    version::Version,
};
use super::water_plane::WaterPlane;
use crate::reader_ext::ReaderExt;

type Objects = (Vec<Model>, Vec<Light>, Vec<Sound>, Vec<Effect>);

#[derive(Debug)]
pub struct RSW {
    pub signature: Box<str>,
    pub version: Version,
    pub flag: u8,
    pub ini_file: Box<str>,
    pub gnd_file: Box<str>,
    pub gat_file: Box<str>,
    pub source_file: Box<str>,
    pub water_configuration: Option<WaterPlane>,
    pub lighting_parameters: LightingParams,
    pub map_boundaries: BoundingBox,
    pub objects: Objects,
    pub quad_tree: QuadTree,
}

impl RSW {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<RSW, Error> {
        let signature = Self::read_signature(reader)?;
        let version = Self::read_version(reader)?;
        let flag = if version >= Version(2, 5, 0) {
            reader.read_u8()?
        } else {
            0
        };

        let ini_file = super::read_euc_kr_string(reader, 40)?;
        let gnd_file = super::read_euc_kr_string(reader, 40)?;
        let gat_file = super::read_euc_kr_string(reader, 40)?;
        let source_file = super::read_euc_kr_string(reader, 40)?;

        let water_configuration = if version < Version(2, 6, 0) {
            Some(WaterPlane::read_single(reader)?)
        } else {
            None
        };
        let lighting_parameters = LightingParams::from_reader(reader)?;

        let map_boundaries = BoundingBox::from_reader(reader)?;

        let objects = Self::read_objects(reader, &version)?;

        let quad_tree = Self::read_quad_tree(reader)?;

        let mut rest = vec![];
        reader.read_to_end(&mut rest)?;
        if !rest.is_empty() {
            return Err(Error::IncompleteRead(version, rest.len()));
        }

        Ok(Self {
            signature,
            version,
            flag,
            ini_file,
            gnd_file,
            gat_file,
            source_file,
            water_configuration,
            lighting_parameters,
            map_boundaries,
            objects,
            quad_tree,
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
        if (*signature).eq("GRSW") {
            Ok(signature)
        } else {
            Err(Error::InvalidSignature(signature))
        }
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let build = if major == 2 && (2..5).contains(&minor) {
            reader.read_u8()? as u32
        } else if major == 2 && (5..7).contains(&minor) {
            reader.read_le_u32()?
        } else {
            0
        };
        let version = Version(major, minor, build);
        if major > 2 || (major == 2 && minor > 6) || (major == 2 && minor == 6 && build > 162) {
            Err(Error::UnknownVersion(version))
        } else {
            Ok(version)
        }
    }

    fn read_objects(mut reader: &mut dyn Read, version: &Version) -> Result<Objects, Error> {
        let count = reader.read_le_u32()?;
        let mut models = vec![];
        let mut lights = vec![];
        let mut sounds = vec![];
        let mut effects = vec![];
        for _ in 0..count {
            let obj_type = reader.read_le_u32()?;
            match obj_type {
                1 => {
                    models.push(Model::from_reader(reader, version)?);
                }
                2 => {
                    lights.push(Light::from_reader(reader, version)?);
                }
                3 => {
                    sounds.push(Sound::from_reader(reader, version)?);
                }
                4 => {
                    effects.push(Effect::from_reader(reader, version)?);
                }
                _ => Err(Error::UnknownObjectType(obj_type))?,
            }
        }
        Ok((models, lights, sounds, effects))
    }

    fn read_quad_tree(mut reader: &mut dyn Read) -> Result<QuadTree, Error> {
        let ranges = (0..QUAD_TREE_SIZE)
            .map(|_| {
                let top = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                let bottom = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                let diameter = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                let center = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                Ok(QuadTreeRange {
                    top,
                    bottom,
                    diameter,
                    center,
                })
            })
            .collect::<Result<Box<[QuadTreeRange]>, Error>>()?;
        Ok(QuadTree { ranges })
    }
}
