use bevy::{
    asset::AssetApp,
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

mod loader;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_loader(loader::AssetLoader);
    }
}

pub fn palette_to_image(palette: &pal::Pal) -> Image {
    let transparency_color = palette.colors[0];

    Image {
        data: palette
            .colors
            .iter()
            .flat_map(|color| {
                if color.alpha > 0
                    || (color.red == transparency_color.red
                        && color.green == transparency_color.green
                        && color.blue == transparency_color.blue)
                {
                    [color.red, color.green, color.blue, color.alpha]
                } else {
                    [color.red, color.green, color.blue, 255]
                }
            })
            .collect(),
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
    }
}
