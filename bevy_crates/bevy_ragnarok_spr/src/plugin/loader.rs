use bevy_asset::{Handle, LoadContext, RenderAssetUsages};
use bevy_image::{Image, ImageSampler};
use bevy_render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use ragnarok_spr::{Error, IndexedSprite, Spr, TrueColorSprite};

use crate::assets::SpriteImages;

pub struct AssetLoader;

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = SpriteImages;
    type Settings = ();
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy_asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let sprite = Spr::from_reader(&mut data.as_slice())?;

        Ok(Self::generate_sprite(load_context, sprite))
    }

    fn extensions(&self) -> &[&str] {
        &["spr"]
    }
}

impl AssetLoader {
    fn generate_sprite(load_context: &mut LoadContext, sprite: Spr) -> SpriteImages {
        let indexed_sprites = Self::load_indexed_sprites(load_context, &sprite.bitmap_images);
        let true_color_sprites =
            Self::load_true_color_sprites(load_context, &sprite.true_color_images);
        let palette =
            load_context.add_labeled_asset("Palette".to_owned(), Image::from(sprite.palette));

        SpriteImages {
            indexed_sprites,
            true_color_sprites,
            palette,
        }
    }

    fn load_indexed_sprites(
        load_context: &mut LoadContext,
        indexed_sprites: &[IndexedSprite],
    ) -> Vec<Handle<Image>> {
        indexed_sprites
            .iter()
            .enumerate()
            .map(|(index, sprite)| {
                let image = Image {
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
                    asset_usage: if cfg!(feature = "debug") {
                        RenderAssetUsages::all()
                    } else {
                        RenderAssetUsages::RENDER_WORLD
                    },
                    ..Default::default()
                };

                load_context.add_labeled_asset(format!("IndexedSprite{}", index), image)
            })
            .collect()
    }

    fn load_true_color_sprites(
        load_context: &mut LoadContext,
        true_color_sprites: &[TrueColorSprite],
    ) -> Vec<Handle<Image>> {
        true_color_sprites
            .iter()
            .enumerate()
            .map(|(index, sprite)| {
                let image = Image {
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
                    asset_usage: if cfg!(feature = "debug") {
                        RenderAssetUsages::all()
                    } else {
                        RenderAssetUsages::RENDER_WORLD
                    },
                    ..Default::default()
                };

                load_context.add_labeled_asset(format!("TrueColorSprite{}", index), image)
            })
            .collect()
    }
}
