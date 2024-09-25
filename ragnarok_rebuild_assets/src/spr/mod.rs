mod error;

use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::{
    common::{Color, Version},
    pal,
};

pub use self::error::Error;

#[derive(Debug)]
pub struct TrueColorSprite {
    pub width: u16,
    pub height: u16,
    pub pixels: Box<[Color]>,
}

#[derive(Debug)]
pub struct IndexedSprite {
    pub width: u16,
    pub height: u16,
    pub indexes: Box<[u8]>,
}

#[derive(Debug)]
pub struct Sprite {
    pub signature: [u8; 2],
    pub version: Version,
    /// Images storing only the index to a palette
    pub bitmap_images: Box<[IndexedSprite]>,
    /// Images storing ABGR bitmaps
    pub true_color_images: Box<[TrueColorSprite]>,
    /// 256 RGBA colors
    pub palette: Option<pal::Palette>,
}

impl Sprite {
    pub fn from_reader(reader: &mut dyn Read) -> Result<Self, Error> {
        let signature = Self::read_signature(reader)?;
        let version = Self::read_version(reader)?;
        let sprite = Self::load_sprite(reader, signature, version)?;

        let mut remainder = vec![];
        reader.read_to_end(&mut remainder)?;

        if !remainder.is_empty() {
            Err(Error::IncompleteRead(remainder.len()))
        } else {
            Ok(sprite)
        }
    }

    fn read_signature(mut reader: &mut dyn Read) -> Result<[u8; 2], Error> {
        let signature = reader.read_array()?;
        if signature.eq(b"SP") {
            Ok(signature)
        } else {
            Err(Error::WrongSignature)
        }
    }

    fn read_version(mut reader: &mut dyn Read) -> Result<Version, Error> {
        let array: [u8; 2] = reader.read_array()?;
        Ok(Version(array[1], array[0], 0))
    }

    fn load_sprite(
        reader: &mut dyn Read,
        signature: [u8; 2],
        version: Version,
    ) -> Result<Sprite, Error> {
        let (bitmap_image_count, truecolor_image_count) = Self::load_image_count(reader, &version)?;
        let bitmap_images = Self::load_bitmap_images(reader, &version, bitmap_image_count)?;
        let true_color_images =
            Self::load_true_color_images(reader, &version, truecolor_image_count)?;
        let palette = Self::load_palette(reader)?;

        Ok(Sprite {
            signature,
            version,
            bitmap_images,
            true_color_images,
            palette,
        })
    }

    fn load_image_count(mut reader: &mut dyn Read, version: &Version) -> Result<(u16, u16), Error> {
        let bitmap_image_count = reader.read_le_u16()?;
        let true_color_image_count = match version {
            Version(1, 1, 0) => 0,
            Version(2, 0, 0) | Version(2, 1, 0) => reader.read_le_u16()?,
            version => Err(Error::UnsupportedVersion(*version))?,
        };

        Ok((bitmap_image_count, true_color_image_count))
    }

    fn load_bitmap_images(
        mut reader: &mut dyn Read,
        version: &Version,
        bitmap_image_count: u16,
    ) -> Result<Box<[IndexedSprite]>, Error> {
        match version {
            Version(1, 1, 0) | Version(2, 0, 0) => (0..bitmap_image_count)
                .map(|_| Self::load_uncompressed_bitmap(&mut reader))
                .collect::<Result<_, _>>(),
            Version(2, 1, 0) => (0..bitmap_image_count)
                .map(|_| Self::load_compressed_bitmap(&mut reader))
                .collect::<Result<_, _>>(),
            version => Err(Error::UnsupportedVersion(*version))?,
        }
    }

    fn load_true_color_images(
        mut reader: &mut dyn Read,
        version: &Version,
        true_color_image_count: u16,
    ) -> Result<Box<[TrueColorSprite]>, Error> {
        match version {
            Version(1, 1, 0) => Ok(Box::new([])),
            Version(2, 0, 0) | Version(2, 1, 0) => (0..true_color_image_count)
                .map(|_| Self::load_truecolor_bitmap(&mut reader))
                .collect::<Result<_, _>>(),
            version => Err(Error::UnsupportedVersion(*version))?,
        }
    }

    fn load_uncompressed_bitmap(mut reader: &mut dyn Read) -> Result<IndexedSprite, Error> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;

        let pixels = reader
            .read_vec(usize::from(width) * usize::from(height))?
            .into_boxed_slice();

        Ok(IndexedSprite {
            width,
            height,
            indexes: pixels,
        })
    }

    fn load_compressed_bitmap(mut reader: &mut dyn Read) -> Result<IndexedSprite, Error> {
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
        if pixels.len().ne(&(usize::from(width) * usize::from(height))) {
            Err(Error::RLE)?
        }

        Ok(IndexedSprite {
            width,
            height,
            indexes: pixels,
        })
    }

    fn load_truecolor_bitmap(mut reader: &mut dyn Read) -> Result<TrueColorSprite, Error> {
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

    fn load_palette(reader: &mut dyn Read) -> Result<Option<pal::Palette>, Error> {
        match pal::Palette::from_reader(reader) {
            Ok(palette) => Ok(Some(palette)),
            Err(pal::Error::Io(io)) => {
                if io.kind() == std::io::ErrorKind::UnexpectedEof {
                    Ok(None)
                } else {
                    Err(io.into())
                }
            }
        }
    }
}
