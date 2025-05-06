mod aabb_ext;

use bevy::{
    image::{TextureAtlasBuilder, TextureAtlasLayout, TextureAtlasSources},
    math::{UVec2, Vec2},
    platform::collections::HashMap,
    prelude::Image,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

pub use self::aabb_ext::AabbExt;

pub fn build_texture_atlas_from_list_of_images(
    images: &[Image],
    format: TextureFormat,
) -> (Image, Vec<Vec2>) {
    let mut texture_atlas_builder = TextureAtlasBuilder::default();
    texture_atlas_builder.format(format);

    for image in images.iter() {
        texture_atlas_builder.add_texture(None, image);
    }

    let (layout, _sources, color_texture_image) = texture_atlas_builder.build().unwrap_or((
        TextureAtlasLayout::new_empty(UVec2::splat(0)),
        TextureAtlasSources {
            texture_ids: HashMap::new(),
        },
        Image::new_fill(
            Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[255, 0, 255, 255],
            format,
            RenderAssetUsages::RENDER_WORLD,
        ),
    ));

    let layout_size = layout.size.as_vec2();
    let texture_uvs = layout
        .textures
        .iter()
        .flat_map(|texture_rect| {
            let rect = texture_rect.as_rect();
            [rect.min / layout_size, (rect.max - rect.min) / layout_size]
        })
        .collect();

    (color_texture_image, texture_uvs)
}
