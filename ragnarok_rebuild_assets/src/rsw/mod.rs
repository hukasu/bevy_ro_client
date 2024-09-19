mod bounding_box;
mod effect;
mod error;
mod light;
mod lighting_params;
mod model;
mod quad_tree;
mod sound;

use std::io::Read;

use super::common::{Version, WaterPlane};
use ragnarok_rebuild_common::reader_ext::ReaderExt;

pub use self::{
    bounding_box::BoundingBox,
    effect::Effect,
    error::Error,
    light::Light,
    lighting_params::LightingParams,
    model::Model,
    quad_tree::{QuadTree, Range, QUAD_TREE_MAX_DEPTH, QUAD_TREE_SIZE},
    sound::Sound,
};

type Objects = (Box<[Model]>, Box<[Light]>, Box<[Sound]>, Box<[Effect]>);

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
    pub models: Box<[Model]>,
    pub lights: Box<[Light]>,
    pub sounds: Box<[Sound]>,
    pub effects: Box<[Effect]>,
    pub quad_tree: QuadTree,
}

impl RSW {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<RSW, Error> {
        let signature = Self::read_signature(reader)?;
        let version = Version::rsw_version_from_reader(reader)?;
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
            Some(WaterPlane::from_reader(reader)?)
        } else {
            None
        };
        let lighting_parameters = LightingParams::from_reader(reader)?;

        let map_boundaries = BoundingBox::from_reader(reader)?;

        let (models, lights, sounds, effects) = Self::read_objects(reader, &version)?;

        let quad_tree = if version >= Version(2, 0, 0) {
            QuadTree::from_reader(reader)?
        } else {
            QuadTree {
                ranges: std::array::from_fn::<Range, QUAD_TREE_SIZE, _>(|_| Range::default())
                    .into(),
            }
        };

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
            models,
            lights,
            sounds,
            effects,
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
        Ok((
            models.into_boxed_slice(),
            lights.into_boxed_slice(),
            sounds.into_boxed_slice(),
            effects.into_boxed_slice(),
        ))
    }
}
