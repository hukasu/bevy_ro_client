//! Builds Ragnarok Online's Spr files to be used in Bevy

pub mod material;
pub mod plugin;

use bevy_asset::{Asset, Assets, Handle};
use bevy_color::Color;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    query::{QueryEntityError, With},
    system::{Query, Res, ResMut, SystemParam},
};
use bevy_image::Image;
use bevy_mesh::MeshTag;
use bevy_pbr::MeshMaterial3d;
use bevy_reflect::Reflect;
use bevy_render::storage::ShaderStorageBuffer;

use crate::{material::SprMaterial, plugin::SpriteBuffers};

/// Holds an handles to the assets generated from a [`Spr`](ragnarok_spr::Spr).
#[derive(Debug, Component, Reflect)]
pub struct Sprite(pub Handle<SpriteImages>);

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
    sprites: Query<'w, 's, &'static MeshTag, With<MeshMaterial3d<SprMaterial>>>,
    sprite_buffers: Res<'w, SpriteBuffers>,
    shader_storage_buffers: ResMut<'w, Assets<ShaderStorageBuffer>>,
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
        let Ok(tag) = usize::try_from(self.sprites.get(entity)?.0) else {
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

        Ok(())
    }

    pub fn change_tint(&mut self, entity: Entity, tint: Color) -> Result<(), QueryEntityError> {
        let Ok(tag) = usize::try_from(self.sprites.get(entity)?.0) else {
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
        let slice = &mut data[start..end];
        let linear = tint.to_linear();
        for channel in [linear.red, linear.green, linear.blue, linear.alpha] {
            slice[start..(start + 4)].swap_with_slice(&mut channel.to_le_bytes());
        }

        Ok(())
    }

    pub fn change_uv_flip(
        &mut self,
        entity: Entity,
        is_flipped: bool,
    ) -> Result<(), QueryEntityError> {
        let Ok(tag) = usize::try_from(self.sprites.get(entity)?.0) else {
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
        data[start..end].swap_with_slice(&mut [0, 0, 0, is_flipped as u8]);

        Ok(())
    }
}
