mod error;
mod indexed;
#[cfg(feature = "bevy")]
pub mod material;
#[cfg(feature = "bevy")]
pub mod plugin;
mod true_color;

use std::io::Read;

use ragnarok_rebuild_common::{Version, reader_ext::ReaderExt};

pub use self::{error::Error, indexed::IndexedSprite, true_color::TrueColorSprite};

#[derive(Debug)]
pub struct Spr {
    pub signature: [u8; 2],
    pub version: Version,
    /// Images storing only the index to a palette
    pub bitmap_images: Box<[IndexedSprite]>,
    /// Images storing ABGR bitmaps
    pub true_color_images: Box<[TrueColorSprite]>,
    /// 256 RGBA colors
    pub palette: ragnarok_pal::Pal,
}

impl Spr {
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
    ) -> Result<Spr, Error> {
        let (bitmap_image_count, truecolor_image_count) = Self::load_image_count(reader, &version)?;
        let bitmap_images = Self::load_bitmap_images(reader, &version, bitmap_image_count)?;
        let true_color_images =
            Self::load_true_color_images(reader, &version, truecolor_image_count)?;
        let palette = Self::load_palette(reader)?;

        Ok(Spr {
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
        (0..bitmap_image_count)
            .map(|_| IndexedSprite::from_reader(&mut reader, version))
            .collect::<Result<_, _>>()
    }

    fn load_true_color_images(
        mut reader: &mut dyn Read,
        version: &Version,
        true_color_image_count: u16,
    ) -> Result<Box<[TrueColorSprite]>, Error> {
        match version {
            Version(1, 1, 0) => Ok(Box::new([])),
            Version(2, 0, 0) | Version(2, 1, 0) => (0..true_color_image_count)
                .map(|_| TrueColorSprite::from_reader(&mut reader))
                .collect::<Result<_, _>>(),
            version => Err(Error::UnsupportedVersion(*version))?,
        }
    }

    fn load_palette(reader: &mut dyn Read) -> Result<ragnarok_pal::Pal, Error> {
        match ragnarok_pal::Pal::from_reader(reader) {
            Ok(palette) => Ok(palette),
            Err(ragnarok_pal::Error::Io(io)) => {
                if io.kind() == std::io::ErrorKind::UnexpectedEof {
                    Err(Error::BrokenPalette)
                } else {
                    Err(io.into())
                }
            }
        }
    }
}
