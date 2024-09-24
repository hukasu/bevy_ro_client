mod error;

use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

pub use self::error::Error;

#[derive(Debug)]
pub struct Palette {
    pub colors: [PaletteColor; 256],
}

#[derive(Debug, Clone, Copy)]
pub struct PaletteColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Palette {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, Error> {
        let palette_contents: [u8; 1024] = reader.read_array()?;
        Ok(Self::from_bytes(&palette_contents))
    }

    pub fn from_bytes(bytes: &[u8; 1024]) -> Self {
        Self {
            colors: std::array::from_fn(|index| PaletteColor {
                red: bytes[index * 4],
                green: bytes[index * 4 + 1],
                blue: bytes[index * 4 + 2],
                alpha: bytes[index * 4 + 3],
            }),
        }
    }
}
