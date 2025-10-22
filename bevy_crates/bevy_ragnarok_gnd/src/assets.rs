//! Assets built from a [`Gnd`](ragnarok_gnd::Gnd)

use std::collections::HashMap;

use bevy_asset::{Asset, Handle};
use bevy_image::Image;
use bevy_math::USizeVec3;
use bevy_reflect::TypePath;
use bevy_scene::Scene;

use crate::material::GndMaterial;

/// Asset built from a [`Gnd`](ragnarok_gnd::Gnd).
#[derive(Asset, TypePath)]
pub struct GndAsset {
    /// Handle to the [`Scene`] representation of the [`Gnd`](ragnarok_gnd::Gnd)
    pub scene: Handle<Scene>,
    /// Handles to texture [`Image`]
    pub textures: Vec<Handle<Image>>,
    /// Handles to texture [`Image`]
    pub materials: HashMap<USizeVec3, Handle<GndMaterial>>,
}
