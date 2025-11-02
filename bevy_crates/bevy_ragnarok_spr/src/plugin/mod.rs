mod loader;

use bevy_asset::{AssetApp, AssetServer, Handle, RenderAssetUsages};
use bevy_ecs::resource::Resource;
use bevy_render::storage::ShaderStorageBuffer;

use crate::{
    Sprite, SpriteImages,
    material::{self, SprMaterial},
};

const INITIAL_SHADER_BUFFER_CAPACITY: usize = 128;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Types
            .register_type::<Sprite>()
            // Asset Loader
            .register_asset_loader(loader::AssetLoader)
            // Asset
            .init_asset::<SpriteImages>()
            .register_asset_reflect::<SpriteImages>()
            // MaterialPlugin
            .add_plugins(material::plugin::Plugin);

        let asset_server = app.world().resource::<AssetServer>();
        let uv_indices = asset_server.add(ShaderStorageBuffer::new(
            &Vec::with_capacity(SprMaterial::UV_INDEX_STRIDE * INITIAL_SHADER_BUFFER_CAPACITY),
            RenderAssetUsages::all(),
        ));
        let tint = asset_server.add(ShaderStorageBuffer::new(
            &Vec::with_capacity(SprMaterial::TINT_STRIDE * INITIAL_SHADER_BUFFER_CAPACITY),
            RenderAssetUsages::all(),
        ));
        let flipped_uvs = asset_server.add(ShaderStorageBuffer::new(
            &Vec::with_capacity(SprMaterial::FLIPPED_UVS_STRIDE * INITIAL_SHADER_BUFFER_CAPACITY),
            RenderAssetUsages::all(),
        ));

        app.insert_resource(SpriteBuffers {
            uv_indices,
            tint,
            flipped_uvs,
        });
    }
}

#[derive(Resource)]
pub(crate) struct SpriteBuffers {
    pub uv_indices: Handle<ShaderStorageBuffer>,
    pub tint: Handle<ShaderStorageBuffer>,
    pub flipped_uvs: Handle<ShaderStorageBuffer>,
}
