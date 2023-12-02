mod error;

use crate::reader_ext::ReaderExt;

pub use self::error::SpriteError;

#[derive(Debug)]
pub struct SpriteABGR {
    pub width: u16,
    pub height: u16,
    pub pixels: Box<[u8]>,
}

#[derive(Debug)]
pub struct SpriteIndexed {
    pub width: u16,
    pub height: u16,
    pub pixels: Box<[u8]>,
}

#[derive(Debug)]
pub struct Sprite {
    header: [u8; 2],
    version: [u8; 2],
    bitmap_image_count: u16,
    truecolor_image_count: u16,
    /// Images storing only the index to a palette
    bitmap_images: Box<[SpriteIndexed]>,
    /// Images storing ABGR bitmaps
    truecolor_image: Box<[SpriteABGR]>,
    /// 256 RGBA colors
    palette: [u8; 1024],
}

impl Sprite {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, SpriteError> {
        let header = bytes.read_array()?;
        let version = bytes.read_array()?;
        if !matches!(header, [b'S', b'P']) {
            Err(SpriteError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid sprite header.",
            )))?
        };

        let bitmap_image_count = bytes.read_le_u16()?;
        let truecolor_image_count = bytes.read_le_u16()?;

        let bitmap_images = match version {
            [0, 2] => (0..bitmap_image_count)
                .map(|_| Self::load_uncompressed_bitmap(&mut bytes))
                .collect::<Result<_, _>>()?,
            [1, 2] => (0..bitmap_image_count)
                .map(|_| Self::load_compressed_bitmap(&mut bytes))
                .collect::<Result<_, _>>()?,
            _ => Err(SpriteError::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unsupported sprite version.",
            )))?,
        };

        let truecolor_image = (0..truecolor_image_count)
            .map(|_| Self::load_truecolor_bitmap(&mut bytes))
            .collect::<Result<_, _>>()?;

        let palette = bytes.read_array()?;

        Ok(Sprite {
            header,
            version,
            bitmap_image_count,
            truecolor_image_count,
            bitmap_images,
            truecolor_image,
            palette,
        })
    }

    fn load_uncompressed_bitmap(reader: &mut &[u8]) -> Result<SpriteIndexed, SpriteError> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;

        let pixels = reader
            .read_vec((width * height) as usize)?
            .into_boxed_slice();

        Ok(SpriteIndexed {
            width,
            height,
            pixels,
        })
    }

    fn load_compressed_bitmap(reader: &mut &[u8]) -> Result<SpriteIndexed, SpriteError> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;
        let compressed_buffer_size = reader.read_le_u16()?;

        let buffer = reader.read_vec(compressed_buffer_size as usize)?;

        let pixels = buffer
            .into_iter()
            .scan(false, |seen_zero, cur| match seen_zero {
                true => {
                    *seen_zero = false;
                    Some(vec![0; cur as usize])
                }
                false => {
                    if cur == 0 {
                        *seen_zero = true;
                        Some(vec![])
                    } else {
                        Some(vec![cur])
                    }
                }
            })
            .flatten()
            .collect::<Box<_>>();
        if pixels.len().ne(&((width * height) as usize)) {
            Err(SpriteError::RLE)?
        }

        Ok(SpriteIndexed {
            width,
            height,
            pixels,
        })
    }

    fn load_truecolor_bitmap(reader: &mut &[u8]) -> Result<SpriteABGR, SpriteError> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;

        let pixels = reader
            .read_vec((width as u32 * height as u32 * 4) as usize)?
            .chunks(4)
            .flat_map(|chunk| chunk.iter().rev().collect::<Vec<_>>())
            .copied()
            .collect();

        Ok(SpriteABGR {
            width,
            height,
            pixels,
        })
    }
}
