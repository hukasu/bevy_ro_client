//! Builds Ragnarok Online's Pal files to be used in Bevy

pub mod plugin;

use bevy_asset::{Handle, RenderAssetUsages};
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_image::{Image, ImageSampler};
use bevy_reflect::Reflect;
use bevy_render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use ragnarok_pal::Pal;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
/// A palette containing 256 colors in a 1D image
pub struct Palette(pub Handle<Image>);

/// Converts a Ragnarok Online's Pal file into a Bevy [`Image`]
pub fn pal_to_image(palette: Pal) -> Image {
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
