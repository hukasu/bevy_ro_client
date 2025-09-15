#[cfg(feature = "bevy")]
pub mod components;
mod error;
#[cfg(feature = "bevy")]
pub mod plugin;
#[cfg(feature = "warning")]
pub mod warnings;

use std::io::Read;

#[cfg(feature = "bevy")]
use bevy_asset::RenderAssetUsages;
#[cfg(feature = "bevy")]
use bevy_image::{Image, ImageSampler};
#[cfg(feature = "bevy")]
use bevy_render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use ragnarok_rebuild_common::{Color, reader_ext::ReaderExt};

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

#[cfg(feature = "bevy")]
impl From<Pal> for Image {
    fn from(palette: Pal) -> Self {
        Image {
            data: Some(
                palette
                    .colors
                    .iter()
                    .flat_map(|color| [color.red, color.green, color.blue, color.alpha])
                    .collect(),
            ),
            texture_descriptor: TextureDescriptor {
                label: Some("palette"),
                size: Extent3d {
                    width: 256,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D1,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            sampler: ImageSampler::nearest(),
            texture_view_descriptor: None,
            asset_usage: if cfg!(feature = "debug") {
                RenderAssetUsages::all()
            } else {
                RenderAssetUsages::RENDER_WORLD
            },
            ..Default::default()
        }
    }
}
