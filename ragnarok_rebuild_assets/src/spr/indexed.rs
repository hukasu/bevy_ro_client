use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

use crate::common::Version;

#[derive(Debug)]
pub struct IndexedSprite {
    pub width: u16,
    pub height: u16,
    pub indexes: Box<[u8]>,
}

impl IndexedSprite {
    pub fn from_reader(reader: &mut dyn Read, version: &Version) -> Result<Self, super::Error> {
        match version {
            Version(1, 1, 0) | Version(2, 0, 0) => Self::load_uncompressed_bitmap(reader),
            Version(2, 1, 0) => Self::load_compressed_bitmap(reader),
            version => Err(super::Error::UnsupportedVersion(*version)),
        }
    }

    fn load_uncompressed_bitmap(mut reader: &mut dyn Read) -> Result<IndexedSprite, super::Error> {
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

    fn load_compressed_bitmap(mut reader: &mut dyn Read) -> Result<IndexedSprite, super::Error> {
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
            Err(super::Error::RLE)?
        }

        Ok(IndexedSprite {
            width,
            height,
            indexes: pixels,
        })
    }
}
