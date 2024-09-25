use bevy::{
    asset::{Asset, AsyncReadExt, Handle},
    prelude::Image,
    reflect::Reflect,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::ImageSampler,
    },
};

use ragnarok_rebuild_assets::spr;

use crate::assets::pal;

#[derive(Debug, Asset, Reflect)]
pub struct Sprite {
    pub indexed_sprites: Vec<Handle<Image>>,
    pub true_color_sprites: Vec<Handle<Image>>,
    pub palette: Option<Handle<Image>>,
}

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Sprite;
    type Settings = ();
    type Error = spr::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let sprite = spr::Sprite::from_reader(&mut data.as_slice())?;

            Ok(Self::generate_sprite(load_context, sprite))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["spr"]
    }
}

impl AssetLoader {
    fn generate_sprite(load_context: &mut bevy::asset::LoadContext, sprite: spr::Sprite) -> Sprite {
        let indexed_sprites = Self::load_indexed_sprites(load_context, &sprite.bitmap_images);
        let true_color_sprites =
            Self::load_true_color_sprites(load_context, &sprite.truecolor_images);
        let palette = if let Some(palette) = sprite.palette {
            let handle = load_context
                .add_labeled_asset("Palette".to_owned(), pal::palette_to_image(&palette));
            Some(handle)
        } else {
            None
        };

        Sprite {
            indexed_sprites,
            true_color_sprites,
            palette,
        }
    }

    fn load_indexed_sprites(
        load_context: &mut bevy::asset::LoadContext,
        indexed_sprites: &[spr::IndexedSprite],
    ) -> Vec<Handle<Image>> {
        indexed_sprites
            .iter()
            .enumerate()
            .map(|(index, sprite)| {
                let image = Image {
                    data: sprite.indexes.to_vec(),
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
                };

                load_context.add_labeled_asset(format!("IndexedSprite{}", index), image)
            })
            .collect()
    }

    fn load_true_color_sprites(
        load_context: &mut bevy::asset::LoadContext,
        true_color_sprites: &[spr::TrueColorSprite],
    ) -> Vec<Handle<Image>> {
        true_color_sprites
            .iter()
            .enumerate()
            .map(|(index, sprite)| {
                let image = Image {
                    data: sprite
                        .pixels
                        .iter()
                        .flat_map(|pixel| [pixel.red, pixel.green, pixel.blue, pixel.alpha])
                        .collect(),
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
                };

                load_context.add_labeled_asset(format!("TrueColorSprite{}", index), image)
            })
            .collect()
    }
}
