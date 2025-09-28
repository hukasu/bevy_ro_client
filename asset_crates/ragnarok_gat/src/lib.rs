mod error;
mod tile;
#[cfg(feature = "warning")]
pub mod warnings;

use std::io::Read;

use ragnarok_rebuild_common::{Version, reader_ext::ReaderExt};

pub use self::{
    error::Error,
    tile::{Tile, TileType},
};

#[derive(Debug)]
pub struct Gat {
    pub signature: [u8; 4],
    pub version: Version,
    pub width: u32,
    pub height: u32,
    pub tiles: Box<[Tile]>,
}

impl Gat {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, Error> {
        let signature = Self::read_signature(reader)?;
        let version = Self::read_version(reader)?;

        let width = reader.read_le_u32()?;
        let height = reader.read_le_u32()?;

        let tiles = Self::read_tiles(reader, width, height)?;

        let mut rest = vec![];
        reader.read_to_end(&mut rest)?;
        if !rest.is_empty() {
            return Err(Error::IncompleteRead(version, rest.len()));
        }

        Ok(Self {
            signature,
            version,
            width,
            height,
            tiles,
        })
    }

    fn read_signature(mut reader: &mut dyn Read) -> Result<[u8; 4], Error> {
        let signature = reader.read_array()?;

        if signature.ne(b"GRAT") {
            Err(Error::InvalidSignature(signature))
        } else {
            Ok(signature)
        }
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, Error> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;

        let version = Version(major, minor, 0);

        match version {
            Version(1, 2, 0) | Version(1, 3, 0) => Ok(version),
            version => Err(Error::UnknownVersion(version)),
        }
    }

    fn read_tiles(reader: &mut dyn Read, width: u32, height: u32) -> Result<Box<[Tile]>, Error> {
        (0..(width * height))
            .map(|_| Tile::from_reader(reader))
            .collect::<Result<Box<[_]>, Error>>()
    }
}
