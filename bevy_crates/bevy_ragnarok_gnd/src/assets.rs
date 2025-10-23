//! Assets built from a [`Gnd`](ragnarok_gnd::Gnd)

use bevy_asset::{Asset, Handle};
use bevy_image::Image;
use bevy_reflect::TypePath;
use bevy_render::storage::ShaderStorageBuffer;
use bevy_scene::Scene;

use crate::material::GndMaterial;

/// Asset built from a [`Gnd`](ragnarok_gnd::Gnd).
#[derive(Asset, TypePath)]
pub struct GndAsset {
    /// Handle to the [`Scene`] representation of the [`Gnd`](ragnarok_gnd::Gnd)
    pub scene: Handle<Scene>,
    /// Handles to texture [`Image`]
    pub textures: Vec<Handle<Image>>,
    /// Handle to [`ShaderStorageBuffer`] built from [`Gnd::surfaces`](ragnarok_gnd::Gnd::surfaces)
    pub surfaces: Handle<ShaderStorageBuffer>,
    /// Materials built from [`Gnd::surfaces`](ragnarok_gnd::Gnd::ground_mesh_cubes).
    pub materials: Vec<GndCubeMaterials>,
}

/// Handles to [`GndMaterial`] built from [`Gnd::surfaces`](ragnarok_gnd::Gnd::ground_mesh_cubes).
pub struct GndCubeMaterials {
    /// Handle to the top surface. Will be [`None`] if
    /// [`GroundMeshCube::upwards_facing_surface`](ragnarok_gnd::GroundMeshCube::upwards_facing_surface)
    /// is `-1`.
    pub up_material: Option<Handle<GndMaterial>>,
    /// Handle to the east surface. Will be [`None`] if
    /// [`GroundMeshCube::east_facing_surface`](ragnarok_gnd::GroundMeshCube::east_facing_surface)
    /// is `-1`.
    pub east_material: Option<Handle<GndMaterial>>,
    /// Handle to the north surface. Will be [`None`] if
    /// [`GroundMeshCube::north_facing_surface`](ragnarok_gnd::GroundMeshCube::north_facing_surface)
    /// is `-1`.
    pub north_material: Option<Handle<GndMaterial>>,
}
