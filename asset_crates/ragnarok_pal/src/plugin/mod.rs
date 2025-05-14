use bevy_asset::{AssetApp, RenderAssetUsages};
use bevy_image::{Image, ImageSampler};
use bevy_render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::Pal;

mod loader;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.register_asset_loader(loader::AssetLoader);
    }
}

pub fn palette_to_image(palette: &Pal) -> Image {
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
    }
}
