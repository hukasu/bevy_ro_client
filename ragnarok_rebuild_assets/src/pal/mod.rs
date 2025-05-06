mod error;

use std::io::Read;

use ragnarok_rebuild_common::{reader_ext::ReaderExt, Color};

pub use self::error::Error;

#[derive(Debug)]
pub struct Pal {
    pub colors: [Color; 256],
}

impl Pal {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, Error> {
        let palette_contents: [u8; 1024] = reader.read_array()?;
        Ok(Self::from_bytes(&palette_contents))
    }

    pub fn from_bytes(bytes: &[u8; 1024]) -> Self {
        Self {
            colors: std::array::from_fn(|index| Color {
                red: bytes[index * 4],
                green: bytes[index * 4 + 1],
                blue: bytes[index * 4 + 2],
                alpha: bytes[index * 4 + 3],
            }),
        }
    }
}
