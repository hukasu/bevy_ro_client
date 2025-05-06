mod animation;
mod error;
mod frame;
mod layer;

use std::io::Read;

use ragnarok_rebuild_common::{reader_ext::ReaderExt, Version};

pub use self::{animation::Animation, error::Error, frame::Frame, layer::Layer};

#[derive(Debug)]
pub struct Imf {
    pub version: Version,
    pub checksum: u32,
    pub max_index: u32,
    pub layers: Box<[Layer]>,
}

impl Imf {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, Error> {
        let version = Self::read_version(reader)?;

        let checksum = reader.read_le_u32()?;

        let max_index = reader.read_le_u32()?;
        let layers = (0..=max_index)
            .map(|_| Layer::from_reader(reader))
            .collect::<Result<_, _>>()?;

        let mut rest = vec![];
        reader.read_to_end(&mut rest)?;
        if !rest.is_empty() {
            return Err(Error::IncompleteRead(version, rest.len()));
        }

        Ok(Self {
            version,
            checksum,
            max_index,
            layers,
        })
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, Error> {
        let version = reader.read_le_f32()?;

        let version = Version(
            version.floor() as u8,
            (version.fract() * 100.).round() as u8,
            0,
        );

        match version {
            Version(1, 1, 0) => Ok(version),
            version => Err(Error::UnknownVersion(version)),
        }
    }
}
