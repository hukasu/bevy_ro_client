use std::fmt::Display;

use bevy_asset::{Handle, LoadContext, RenderAssetUsages};
use bevy_image::{Image, ImageSampler, TextureAtlasBuilder, TextureAtlasBuilderError};
use bevy_ragnarok_pal::pal_to_image;
use bevy_render::{
    render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
    storage::ShaderStorageBuffer,
};
use ragnarok_spr::{Error, IndexedSprite, Spr, TrueColorSprite};

use crate::SpriteImages;

pub struct AssetLoader;

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = SpriteImages;
    type Settings = ();
    type Error = AssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let sprite = Spr::from_reader(&mut data.as_slice())?;

        Ok(Self::generate_sprite(load_context, sprite)?)
    }

    fn extensions(&self) -> &[&str] {
        &["spr"]
    }
}

impl AssetLoader {
    fn generate_sprite(
        load_context: &mut LoadContext,
        sprite: Spr,
    ) -> Result<SpriteImages, TextureAtlasBuilderError> {
        let (indexed_sprites, indexed_sprites_uvs) =
            Self::load_indexed_sprites(load_context, &sprite.bitmap_images)?;
        let (true_color_sprites, true_color_sprites_uvs) =
            Self::load_true_color_sprites(load_context, &sprite.true_color_images)?;
        let palette =
            load_context.add_labeled_asset("Palette".to_owned(), pal_to_image(sprite.palette));

        Ok(SpriteImages {
            indexed_sprites,
            indexed_sprites_uvs,
            true_color_sprites,
            true_color_sprites_uvs,
            palette,
        })
    }

    fn load_indexed_sprites(
        load_context: &mut LoadContext,
        indexed_sprites: &[IndexedSprite],
    ) -> Result<(Handle<Image>, Handle<ShaderStorageBuffer>), TextureAtlasBuilderError> {
        let mut texture_atlas = TextureAtlasBuilder::default();
        texture_atlas.auto_format_conversion(false);
        texture_atlas.format(TextureFormat::R8Uint);

        let asset_usage = if cfg!(feature = "debug") {
            RenderAssetUsages::all()
        } else {
            RenderAssetUsages::RENDER_WORLD
        };

        let images = indexed_sprites
            .iter()
            .map(|sprite| Image {
                data: Some(sprite.indexes.to_vec()),
                texture_descriptor: TextureDescriptor {
                    label: Some("indexed_sprite"),
                    size: Extent3d {
                        width: u32::from(sprite.width),
                        height: u32::from(sprite.height),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::R8Uint,
                    usage: TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                sampler: ImageSampler::Default,
                texture_view_descriptor: None,
                asset_usage,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        for image in &images {
            texture_atlas.add_texture(None, image);
        }

        let (texture_layout, _, image) = texture_atlas.build()?;

        let mut uvs_buffer = Vec::with_capacity(indexed_sprites.len() * SpriteImages::UV_STRIDE);
        #[cfg(debug_assertions)]
        let initial_size = uvs_buffer.capacity();

        let total_size = texture_layout.size;
        for t in &texture_layout.textures {
            uvs_buffer.extend_from_slice(&(t.min.x as f32 / total_size.x as f32).to_le_bytes());
            uvs_buffer.extend_from_slice(&(t.min.y as f32 / total_size.y as f32).to_le_bytes());
            uvs_buffer.extend_from_slice(&(t.max.x as f32 / total_size.x as f32).to_le_bytes());
            uvs_buffer.extend_from_slice(&(t.max.y as f32 / total_size.y as f32).to_le_bytes());
        }

        #[cfg(debug_assertions)]
        debug_assert_eq!(initial_size, uvs_buffer.len());

        Ok((
            load_context.add_labeled_asset("IndexedTextureAtlas".to_owned(), image),
            load_context.add_labeled_asset(
                "IndexedTextureAtlasUvs".to_owned(),
                ShaderStorageBuffer::new(&uvs_buffer, asset_usage),
            ),
        ))
    }

    fn load_true_color_sprites(
        load_context: &mut LoadContext,
        true_color_sprites: &[TrueColorSprite],
    ) -> Result<(Handle<Image>, Handle<ShaderStorageBuffer>), TextureAtlasBuilderError> {
        let mut texture_atlas = TextureAtlasBuilder::default();
        texture_atlas.auto_format_conversion(false);
        texture_atlas.format(TextureFormat::Rgba8UnormSrgb);

        let asset_usage = if cfg!(feature = "debug") {
            RenderAssetUsages::all()
        } else {
            RenderAssetUsages::RENDER_WORLD
        };

        let images = true_color_sprites
            .iter()
            .map(|sprite| Image {
                data: Some(
                    sprite
                        .pixels
                        .iter()
                        .flat_map(|pixel| [pixel.red, pixel.green, pixel.blue, pixel.alpha])
                        .collect(),
                ),
                texture_descriptor: TextureDescriptor {
                    label: Some("true_color_sprite"),
                    size: Extent3d {
                        width: u32::from(sprite.width),
                        height: u32::from(sprite.height),
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba8UnormSrgb,
                    usage: TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                sampler: ImageSampler::Default,
                texture_view_descriptor: None,
                asset_usage,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        for image in &images {
            texture_atlas.add_texture(None, image);
        }

        let (texture_layout, _, image) = texture_atlas.build()?;

        let mut uvs_buffer = Vec::with_capacity(true_color_sprites.len() * SpriteImages::UV_STRIDE);
        #[cfg(debug_assertions)]
        let initial_size = uvs_buffer.capacity();

        let total_size = texture_layout.size;
        for t in &texture_layout.textures {
            uvs_buffer.extend_from_slice(&(t.min.x as f32 / total_size.x as f32).to_le_bytes());
            uvs_buffer.extend_from_slice(&(t.min.y as f32 / total_size.y as f32).to_le_bytes());
            uvs_buffer.extend_from_slice(&(t.max.x as f32 / total_size.x as f32).to_le_bytes());
            uvs_buffer.extend_from_slice(&(t.max.y as f32 / total_size.y as f32).to_le_bytes());
        }

        #[cfg(debug_assertions)]
        debug_assert_eq!(initial_size, uvs_buffer.len());

        Ok((
            load_context.add_labeled_asset("IndexedTextureAtlas".to_owned(), image),
            load_context.add_labeled_asset(
                "IndexedTextureAtlasUvs".to_owned(),
                ShaderStorageBuffer::new(&uvs_buffer, asset_usage),
            ),
        ))
    }
}

#[derive(Debug)]
pub enum AssetLoaderError {
    Spr(Error),
    TextureAtlas(TextureAtlasBuilderError),
}

impl Display for AssetLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLoaderError::Spr(err) => writeln!(f, "Could not build Spr due to {err}."),
            AssetLoaderError::TextureAtlas(err) => {
                writeln!(f, "Could not build Spr texture atlas due to {err}.")
            }
        }
    }
}

impl std::error::Error for AssetLoaderError {}

impl From<Error> for AssetLoaderError {
    fn from(value: Error) -> Self {
        Self::Spr(value)
    }
}

impl From<std::io::Error> for AssetLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Spr(value.into())
    }
}

impl From<TextureAtlasBuilderError> for AssetLoaderError {
    fn from(value: TextureAtlasBuilderError) -> Self {
        Self::TextureAtlas(value)
    }
}
