//! Assets built from a [`Gnd`](ragnarok_gnd::Gnd)

use bevy_asset::{Asset, Handle};
use bevy_mesh::Mesh;
use bevy_reflect::TypePath;
use bevy_scene::Scene;

use crate::material::GndMaterial;

/// Asset built from a [`Gnd`](ragnarok_gnd::Gnd).
#[derive(Asset, TypePath)]
pub struct GndAsset {
    /// Handle to the [`Mesh`] built from the textures of a [`Gnd`](ragnarok_gnd::Gnd)
    pub mesh: Handle<Mesh>,
    /// Handle to the [`GndMaterial`] built from the textures of a [`Gnd`](ragnarok_gnd::Gnd)
    pub material: Handle<GndMaterial>,
    /// Handle to the [`Scene`] representation of the [`Gnd`](ragnarok_gnd::Gnd)
    pub scene: Handle<Scene>,
}
