//! Builds Ragnarok Online's Spr files to be used in Bevy

pub mod material;
pub mod plugin;

use bevy_app::AppExit;
use bevy_asset::{Asset, Assets, Handle};
use bevy_color::Color;
use bevy_derive::Deref;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    lifecycle::HookContext,
    query::QueryEntityError,
    system::{Query, Res, ResMut, SystemParam},
    world::DeferredWorld,
};
use bevy_image::Image;
use bevy_log::error;
use bevy_mesh::MeshTag;
use bevy_pbr::MeshMaterial3d;
use bevy_reflect::Reflect;
use bevy_render::storage::ShaderStorageBuffer;

use crate::{
    material::SprMaterial,
    plugin::{SpriteBuffers, SpriteTags},
};

/// Holds an handles to the assets generated from a [`Spr`](ragnarok_spr::Spr).
///
/// Entities holding this [`Component`] are given a [`MeshTag`]
/// and [`MeshMaterial3d`].
#[derive(Debug, Reflect, Component, Deref)]
#[component(immutable, on_add=Self::on_add_hook, on_remove=Self::on_remove_hook)]
pub struct Sprite(Handle<SpriteImages>);

impl Sprite {
    fn on_add_hook(mut deferred_world: DeferredWorld, hook_context: HookContext) {
        let entity = hook_context.entity;

        let mut tags = deferred_world.resource_mut::<SpriteTags>();

        let (new_tag, increase_buffers) = if let Some(new_tag) = tags.free.pop() {
            (new_tag, false)
        } else {
            let new_tag = tags.last;
            tags.last += 1;
            (new_tag, true)
        };

        if increase_buffers {
            let buffers = deferred_world.resource::<SpriteBuffers>().clone();
            let mut shader_buffers = deferred_world.resource_mut::<Assets<ShaderStorageBuffer>>();

            let Some(uv_indexes) = shader_buffers
                .get_mut(buffers.uv_indices.id())
                .and_then(|buffer| buffer.data.as_mut())
            else {
                unreachable!("SpriteBuffer handles must be valid.");
            };
            uv_indexes.extend_from_slice(&0u32.to_le_bytes());

            let Some(tints) = shader_buffers
                .get_mut(buffers.tint.id())
                .and_then(|buffer| buffer.data.as_mut())
            else {
                unreachable!("SpriteBuffer handles must be valid.");
            };
            for _ in 0..4 {
                tints.extend_from_slice(&1f32.to_le_bytes());
            }

            let Some(flipped_uvs) = shader_buffers
                .get_mut(buffers.flipped_uvs.id())
                .and_then(|buffer| buffer.data.as_mut())
            else {
                unreachable!("SpriteBuffer handles must be valid.");
            };
            flipped_uvs.extend_from_slice(&0u32.to_le_bytes());
        }

        deferred_world
            .commands()
            .entity(entity)
            .insert(MeshTag(new_tag));
    }

    fn on_remove_hook(mut deferred_world: DeferredWorld, hook_context: HookContext) {
        let entity = hook_context.entity;

        let Some(freed_tag) = deferred_world
            .entity(entity)
            .get::<MeshTag>()
            .map(|tag| **tag)
        else {
            error!("MeshTag of {entity} was removed, breaking invariant.");
            deferred_world.write_message(AppExit::from_code(1));
            return;
        };

        let mut tags = deferred_world.resource_mut::<SpriteTags>();
        tags.free.push(freed_tag);
    }
}

impl From<Handle<SpriteImages>> for Sprite {
    fn from(value: Handle<SpriteImages>) -> Self {
        Self(value)
    }
}

/// Assets generated from a [`Spr`](ragnarok_spr::Spr).
#[derive(Debug, Asset, Reflect)]
pub struct SpriteImages {
    /// Texture atlas with indices into the palette
    pub indexed_sprites: Handle<Image>,
    /// Uvs into the indices texture atlas
    pub indexed_sprites_uvs: Handle<ShaderStorageBuffer>,
    /// True color sprite texture atlas
    pub true_color_sprites: Handle<Image>,
    /// Uvs into the true color texture atlas
    pub true_color_sprites_uvs: Handle<ShaderStorageBuffer>,
    /// Default sprite palette
    pub palette: Handle<Image>,
}

impl SpriteImages {
    /// Stride of each item in the Uvs [`ShaderStorageBuffer`]
    pub const UV_STRIDE: usize = 4 * 4;
}

#[derive(SystemParam)]
pub struct SpritesManager<'w, 's> {
    sprites: Query<'w, 's, (&'static MeshTag, &'static mut MeshMaterial3d<SprMaterial>)>,
    sprite_buffers: Res<'w, SpriteBuffers>,
    shader_storage_buffers: ResMut<'w, Assets<ShaderStorageBuffer>>,
    // Work around https://github.com/bevyengine/bevy/issues/20378
    mesh_materials: ResMut<'w, Assets<SprMaterial>>,
}

impl SpritesManager<'_, '_> {
    pub fn update_sprite(
        &mut self,
        entity: Entity,
        index: u32,
        true_color: bool,
        tint: Color,
        is_flipped: bool,
    ) -> Result<(), QueryEntityError> {
        self.change_image(entity, index, true_color)?;
        self.change_tint(entity, tint)?;
        self.change_uv_flip(entity, is_flipped)
    }

    pub fn change_image(
        &mut self,
        entity: Entity,
        mut index: u32,
        true_color: bool,
    ) -> Result<(), QueryEntityError> {
        let (mesh_tag, mesh_material) = self.sprites.get(entity)?;
        let Ok(tag) = usize::try_from(mesh_tag.0) else {
            unreachable!("Tag should never exceed usize.");
        };

        let Some(buffer) = self
            .shader_storage_buffers
            .get_mut(self.sprite_buffers.uv_indices.id())
        else {
            unreachable!("Handles of SpriteBuffers must be valid.");
        };

        if true_color {
            index |= 0x80000000;
        }

        let start = tag * SprMaterial::UV_INDEX_STRIDE;
        let end = (tag + 1) * SprMaterial::UV_INDEX_STRIDE;

        let Some(data) = buffer.data.as_mut() else {
            unreachable!("Buffer data must be available on the CPU.");
        };
        data[start..end].swap_with_slice(&mut index.to_le_bytes());

        self.mesh_materials.get_mut(mesh_material.id());

        Ok(())
    }

    pub fn change_tint(&mut self, entity: Entity, tint: Color) -> Result<(), QueryEntityError> {
        let (mesh_tag, mesh_material) = self.sprites.get(entity)?;
        let Ok(tag) = usize::try_from(mesh_tag.0) else {
            unreachable!("Tag should never exceed usize.");
        };

        let Some(buffer) = self
            .shader_storage_buffers
            .get_mut(self.sprite_buffers.tint.id())
        else {
            unreachable!("Handles of SpriteBuffers must be valid.");
        };

        let start = tag * SprMaterial::TINT_STRIDE;
        let end = (tag + 1) * SprMaterial::TINT_STRIDE;

        let Some(data) = buffer.data.as_mut() else {
            unreachable!("Buffer data must be available on the CPU.");
        };
        let mut slice = &mut data[start..end];
        let linear = tint.to_linear();
        for channel in [linear.red, linear.green, linear.blue, linear.alpha] {
            slice[0..4].swap_with_slice(&mut channel.to_le_bytes());
            slice = &mut slice[4..];
        }
        debug_assert!(slice.is_empty());

        self.mesh_materials.get_mut(mesh_material.id());

        Ok(())
    }

    pub fn change_uv_flip(
        &mut self,
        entity: Entity,
        is_flipped: bool,
    ) -> Result<(), QueryEntityError> {
        let (mesh_tag, mesh_material) = self.sprites.get(entity)?;
        let Ok(tag) = usize::try_from(mesh_tag.0) else {
            unreachable!("Tag should never exceed usize.");
        };

        let Some(buffer) = self
            .shader_storage_buffers
            .get_mut(self.sprite_buffers.flipped_uvs.id())
        else {
            unreachable!("Handles of SpriteBuffers must be valid.");
        };

        let start = tag * SprMaterial::FLIPPED_UVS_STRIDE;
        let end = (tag + 1) * SprMaterial::FLIPPED_UVS_STRIDE;

        let Some(data) = buffer.data.as_mut() else {
            unreachable!("Buffer data must be available on the CPU.");
        };
        data[start..end].swap_with_slice(&mut (is_flipped as u32).to_le_bytes());

        self.mesh_materials.get_mut(mesh_material.id());

        Ok(())
    }
}
