use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::common::Color;

#[derive(Debug)]
pub struct TrueColorSprite {
    pub width: u16,
    pub height: u16,
    pub pixels: Box<[Color]>,
}

impl TrueColorSprite {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<TrueColorSprite, super::Error> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;

        let pixels = reader
            .read_vec(usize::from(width) * usize::from(height) * 4)?
            .chunks(4)
            .map(|chunk| Color {
                red: chunk[3],
                green: chunk[2],
                blue: chunk[1],
                alpha: chunk[0],
            })
            .collect();

        Ok(TrueColorSprite {
            width,
            height,
            pixels,
        })
    }
}
