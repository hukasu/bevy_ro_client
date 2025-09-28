mod bounding_box;
mod effect;
mod error;
mod light;
mod lighting_params;
mod model;
mod quad_tree;
mod sound;
#[cfg(feature = "warning")]
pub mod warnings;

use std::io::Read;

use ragnarok_rebuild_common::{
    Version, WaterPlane, euc_kr::read_euc_kr_string, reader_ext::ReaderExt,
};

pub use self::{
    bounding_box::BoundingBox,
    effect::Effect,
    error::Error,
    light::Light,
    lighting_params::LightingParams,
    model::Model,
    quad_tree::{QUAD_TREE_MAX_DEPTH, QUAD_TREE_SIZE, QuadTree, Range},
    sound::Sound,
};

type Objects = (Box<[Model]>, Box<[Light]>, Box<[Sound]>, Box<[Effect]>);

#[derive(Debug)]
pub struct Rsw {
    pub signature: [u8; 4],
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

impl Rsw {
    pub fn from_reader(reader: &mut dyn Read) -> Result<Rsw, Error> {
        let signature = Self::read_signature(reader)?;
        let version = Self::read_version(reader)?;

        match version {
            Version(1, 9, 0)
            | Version(2, 1, 0)
            | Version(2, 2, 0)
            | Version(2, 2, 1)
            | Version(2, 3, 1)
            | Version(2, 4, 1)
            | Version(2, 5, 36)
            | Version(2, 5, 50)
            | Version(2, 5, 55)
            | Version(2, 5, 131)
            | Version(2, 5, 143)
            | Version(2, 5, 146)
            | Version(2, 6, 161)
            | Version(2, 6, 162)
            | Version(2, 6, 187) => (),
            version => return Err(Error::UnknownVersion(version)),
        };

        let flag = Self::read_flag(reader, &version)?;

        let ini_file = read_euc_kr_string(reader, 40)?;
        let gnd_file = read_euc_kr_string(reader, 40)?;
        let gat_file = read_euc_kr_string(reader, 40)?;
        let source_file = read_euc_kr_string(reader, 40)?;

        let water_configuration = Self::read_water_configuration(reader, &version)?;
        let lighting_parameters = LightingParams::from_reader(reader)?;

        let map_boundaries = BoundingBox::from_reader(reader)?;

        let (models, lights, sounds, effects) = Self::read_objects(reader, &version)?;

        let quad_tree = Self::read_quad_tree(reader, &version)?;

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

    fn read_signature(mut reader: &mut dyn Read) -> Result<[u8; 4], Error> {
        let signature = reader.read_array()?;
        if signature.eq(b"GRSW") {
            Ok(signature)
        } else {
            Err(Error::InvalidSignature)
        }
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, std::io::Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        let build = if major == 2 && (2..5).contains(&minor) {
            u32::from(reader.read_u8()?)
        } else if major == 2 && (5..7).contains(&minor) {
            reader.read_le_u32()?
        } else {
            0
        };
        Ok(Version(major, minor, build))
    }

    fn read_flag(mut reader: &mut dyn Read, version: &Version) -> Result<u8, std::io::Error> {
        match version {
            Version(2, 5, _) | Version(2, 6, _) => reader.read_u8(),
            _ => Ok(0),
        }
    }

    fn read_water_configuration(
        reader: &mut dyn Read,
        version: &Version,
    ) -> Result<Option<WaterPlane>, std::io::Error> {
        match version {
            Version(2, 6, _) => Ok(None),
            _ => Ok(Some(WaterPlane::from_reader(reader)?)),
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
                    lights.push(Light::from_reader(reader)?);
                }
                3 => {
                    sounds.push(Sound::from_reader(reader, version)?);
                }
                4 => {
                    effects.push(Effect::from_reader(reader)?);
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

    fn read_quad_tree(reader: &mut dyn Read, version: &Version) -> Result<QuadTree, Error> {
        match version {
            Version(2, 1, 0)
            | Version(2, 2, 0)
            | Version(2, 2, _)
            | Version(2, 3, _)
            | Version(2, 4, _)
            | Version(2, 5, _)
            | Version(2, 6, _) => QuadTree::from_reader(reader),
            _ => Ok(QuadTree {
                ranges: std::array::from_fn::<Range, QUAD_TREE_SIZE, _>(|_| Range::default())
                    .into(),
            }),
        }
    }
}
