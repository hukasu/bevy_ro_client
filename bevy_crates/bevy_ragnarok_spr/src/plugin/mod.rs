mod loader;

use bevy_app::PreUpdate;
use bevy_asset::{AssetApp, AssetServer, Assets, Handle, RenderAssetUsages};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Commands, Populated, Res, ResMut},
};
use bevy_log::trace;
use bevy_mesh::MeshTag;
use bevy_pbr::MeshMaterial3d;
use bevy_platform::collections::HashMap;
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

        app.init_resource::<SpriteCache>();
        app.init_resource::<SpriteTags>();

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

        app.add_systems(PreUpdate, (add_spr_material, remove_spr_material).chain());
    }
}

#[derive(Default, Resource, Deref, DerefMut)]
pub(crate) struct SpriteCache {
    cache: HashMap<Handle<SpriteImages>, Handle<SprMaterial>>,
}

#[derive(Resource)]
pub(crate) struct SpriteBuffers {
    pub uv_indices: Handle<ShaderStorageBuffer>,
    pub tint: Handle<ShaderStorageBuffer>,
    pub flipped_uvs: Handle<ShaderStorageBuffer>,
}

#[derive(Default, Resource)]
pub(crate) struct SpriteTags {
    pub last: u32,
    pub free: Vec<u32>,
}

#[expect(clippy::type_complexity, reason = "Queries are complex")]
fn add_spr_material(
    mut commands: Commands,
    sprites: Populated<(Entity, &Sprite), (With<MeshTag>, Without<MeshMaterial3d<SprMaterial>>)>,
    mut sprite_cache: ResMut<SpriteCache>,
    sprite_buffers: Res<SpriteBuffers>,
    sprite_images: Res<Assets<SpriteImages>>,
    mut sprite_materials: ResMut<Assets<SprMaterial>>,
) {
    for (entity, sprite) in sprites.into_inner() {
        let material = if let Some(material) = sprite_cache.get(&**sprite) {
            material.clone()
        } else if let Some(images) = sprite_images.get(sprite.id()) {
            let material = SprMaterial {
                index_image: images.indexed_sprites.clone(),
                palette: images.palette.clone(),
                true_color_image: images.true_color_sprites.clone(),
                indexed_uvs: images.indexed_sprites_uvs.clone(),
                true_color_uvs: images.true_color_sprites_uvs.clone(),
                uv_index: sprite_buffers.uv_indices.clone(),
                tint: sprite_buffers.tint.clone(),
                flipped_uvs: sprite_buffers.flipped_uvs.clone(),
            };
            let handle = sprite_materials.add(material);
            sprite_cache.insert((**sprite).clone(), handle.clone());
            handle
        } else {
            continue;
        };

        trace!("Adding MeshMaterial3d to {entity}.");
        commands.entity(entity).insert(MeshMaterial3d(material));
    }
}

fn remove_spr_material(
    mut commands: Commands,
    sprites: Populated<Entity, (With<MeshMaterial3d<SprMaterial>>, Without<Sprite>)>,
) {
    for entity in sprites.into_inner() {
        trace!("Removing MeshMaterial3d from {entity}");
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<SprMaterial>>();
    }
}
