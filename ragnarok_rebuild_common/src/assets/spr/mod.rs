mod asset_loader;
mod error;

use bevy::{asset::Asset, reflect::TypePath, render::texture::Image};

use crate::assets::reader_ext::ReaderExt;

pub use self::{asset_loader::SpriteAssetLoader, error::SpriteError};

#[derive(Debug, Asset, TypePath)]
pub struct Sprite {
    header: [u8; 2],
    version: [u8; 2],
    bitmap_image_count: u16,
    truecolor_image_count: u16,
    bitmap_images: Box<[Image]>,
    truecolor_image: Box<[Image]>,
    palletes: Image,
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

        let palletes = bevy::render::texture::Image::new(
            wgpu::Extent3d {
                width: 256,
                height: 1,
                depth_or_array_layers: 1,
            },
            wgpu::TextureDimension::D1,
            bytes.read_vec(1024)?,
            wgpu::TextureFormat::Rgba8Uint,
        );

        Ok(Sprite {
            header,
            version,
            bitmap_image_count,
            truecolor_image_count,
            bitmap_images,
            truecolor_image,
            palletes,
        })
    }

    fn load_uncompressed_bitmap(
        reader: &mut &[u8],
    ) -> Result<bevy::render::texture::Image, SpriteError> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;

        let buffer = reader.read_vec((width * height) as usize)?;

        Ok(bevy::render::texture::Image::new(
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureDimension::D2,
            buffer,
            wgpu::TextureFormat::R8Uint,
        ))
    }

    fn load_compressed_bitmap(
        reader: &mut &[u8],
    ) -> Result<bevy::render::texture::Image, SpriteError> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;
        let compressed_buffer_size = reader.read_le_u16()?;

        let buffer = reader.read_vec(compressed_buffer_size as usize)?;

        let decompressed = buffer
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
            .collect::<Vec<_>>();
        if decompressed.len().ne(&((width * height) as usize)) {
            Err(SpriteError::RLE)?
        }

        Ok(bevy::render::texture::Image::new(
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureDimension::D2,
            decompressed,
            wgpu::TextureFormat::R8Uint,
        ))
    }

    fn load_truecolor_bitmap(
        reader: &mut &[u8],
    ) -> Result<bevy::render::texture::Image, SpriteError> {
        let width = reader.read_le_u16()?;
        let height = reader.read_le_u16()?;

        let buffer = reader
            .read_vec((width * height * 4) as usize)?
            .chunks(4)
            .flat_map(|chunk| chunk.iter().rev().collect::<Vec<_>>())
            .copied()
            .collect();

        Ok(bevy::render::texture::Image::new(
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            wgpu::TextureDimension::D2,
            buffer,
            wgpu::TextureFormat::Rgba8Uint,
        ))
    }
}
