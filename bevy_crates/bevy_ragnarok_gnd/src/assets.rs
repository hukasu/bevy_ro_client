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
    /// Handle to [`ShaderStorageBuffer`] built from
    /// [`GroundMeshCubes::upwards_facing_surface`](ragnarok_gnd::GroundMeshCube::upwards_facing_surface),
    /// [`GroundMeshCubes::east_facing_surface`](ragnarok_gnd::GroundMeshCube::east_facing_surface), and
    /// [`GroundMeshCubes::north_facing_surface`](ragnarok_gnd::GroundMeshCube::north_facing_surface),
    /// in this order.
    pub surface_ids: Handle<ShaderStorageBuffer>,
    /// Handle to [`ShaderStorageBuffer`] built from [`Gnd::surfaces`](ragnarok_gnd::Gnd::surfaces)
    pub surfaces: Handle<ShaderStorageBuffer>,
    /// Handle to [`ShaderStorageBuffer`] built from
    /// [`GroundMeshCubes`](ragnarok_gnd::GroundMeshCube) faces.
    pub cube_faces: Handle<ShaderStorageBuffer>,
    /// Handle to [`ShaderStorageBuffer`] built from
    /// [`GroundMeshCubes`](ragnarok_gnd::GroundMeshCube) faces' normals.
    pub normals: Handle<ShaderStorageBuffer>,
    /// Materials built from [`Gnd::textures`](ragnarok_gnd::Gnd::textures).
    pub materials: Vec<Handle<GndMaterial>>,
}
