use std::io::Read;

use ragnarok_rebuild_common::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct Lightmap {
    pub pixel_format: i32,
    pub width: u32,
    pub height: u32,
    pub shadow_map_pixels: Box<[Box<[u8]>]>,
    pub light_map_pixels: Box<[Box<[u8]>]>,
}

impl Lightmap {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<Self, super::Error> {
        let lightmap_count = reader.read_le_u32()?;
        let width = reader.read_le_u32()?;
        let height = reader.read_le_u32()?;
        let pixel_format = reader.read_le_i32()?;

        let shadow_map_pixels = (0..(lightmap_count))
            .map(|_| Self::read_shadowmap_data(reader, width, height))
            .collect::<Result<Box<[_]>, super::Error>>()?;
        let light_map_pixels = (0..(lightmap_count))
            .map(|_| Self::read_lightmap_data(reader, width, height))
            .collect::<Result<Box<[_]>, super::Error>>()?;

        Ok(Self {
            pixel_format,
            width,
            height,
            shadow_map_pixels,
            light_map_pixels,
        })
    }

    fn read_shadowmap_data(
        mut reader: &mut dyn Read,
        width: u32,
        height: u32,
    ) -> Result<Box<[u8]>, super::Error> {
        (0..(width * height))
            .map(|_| reader.read_u8().map_err(super::Error::from))
            .collect::<Result<Box<[_]>, super::Error>>()
    }

    fn read_lightmap_data(
        mut reader: &mut dyn Read,
        width: u32,
        height: u32,
    ) -> Result<Box<[u8]>, super::Error> {
        Ok((0..(width * height))
            .map(|_| Ok([reader.read_u8()?, reader.read_u8()?, reader.read_u8()?]))
            .collect::<Result<Box<[_]>, super::Error>>()?
            .iter()
            .flatten()
            .copied()
            .collect())
    }
}
