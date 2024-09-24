use bevy::{
    asset::AsyncReadExt,
    prelude::Image,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        texture::ImageSampler,
    },
};
use ragnarok_rebuild_assets::pal;

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Image;
    type Settings = ();
    type Error = pal::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> impl bevy::utils::ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let pal = pal::Palette::from_reader(&mut data.as_slice())?;

            let transparency_color = pal.colors[0];

            let image = Image {
                data: pal
                    .colors
                    .iter()
                    .flat_map(|color| {
                        if color.alpha > 0
                            || (color.red == transparency_color.red
                                && color.green == transparency_color.green
                                && color.blue == transparency_color.blue)
                        {
                            [
                                (color.red as f32 / 255.).to_le_bytes(),
                                (color.green as f32 / 255.).to_le_bytes(),
                                (color.blue as f32 / 255.).to_le_bytes(),
                                (color.alpha as f32 / 255.).to_le_bytes(),
                            ]
                        } else {
                            [
                                (color.red as f32 / 255.).to_le_bytes(),
                                (color.green as f32 / 255.).to_le_bytes(),
                                (color.blue as f32 / 255.).to_le_bytes(),
                                (1.0f32).to_le_bytes(),
                            ]
                        }
                        .into_iter()
                        .flatten()
                    })
                    .collect(),
                texture_descriptor: TextureDescriptor {
                    label: Some("palette"),
                    size: Extent3d {
                        width: 8,
                        height: 8,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba32Float,
                    usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                sampler: ImageSampler::nearest(),
                texture_view_descriptor: None,
                asset_usage: if cfg!(feature = "debug") {
                    RenderAssetUsages::all()
                } else {
                    RenderAssetUsages::RENDER_WORLD
                },
            };

            Ok(image)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pal"]
    }
}
