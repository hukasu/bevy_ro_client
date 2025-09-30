mod loader;

use bevy_app::PostUpdate;
use bevy_asset::AssetApp;
use bevy_ecs::{
    name::NameOrEntity,
    query::{With, Without},
    schedule::IntoScheduleConfigs,
    system::{Commands, Populated},
};
use bevy_mesh::Mesh3d;
use bevy_pbr::MeshMaterial3d;
use bevy_transform::{TransformSystems, components::GlobalTransform};
use loader::AssetLoader;

use crate::{Model, RsmMaterials, assets::RsmModel, materials::RsmMaterial};

pub struct Plugin {
    pub texture_path_prefix: std::path::PathBuf,
}

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Assets
        app.init_asset::<RsmModel>()
            .register_asset_reflect::<RsmModel>()
            .register_asset_loader(AssetLoader::new(self.texture_path_prefix.clone()));

        // Materials
        app.add_plugins(crate::materials::Plugin);

        // Systems
        app.add_systems(
            PostUpdate,
            invert_material.after(TransformSystems::Propagate),
        );

        // Types
        app.register_type::<Model>();
        app.register_type::<RsmMaterials>();

        #[cfg(feature = "debug")]
        app.add_plugins(crate::debug::Plugin);
    }
}

/// Insert material for models based on the number of negative scale axis.
/// Even number of negative scale axis uses [`RsmMaterials::base`]. Odd uses
/// [`RsmMaterials::inverted`].
#[expect(clippy::type_complexity, reason = "Queries are complex")]
fn invert_material(
    mut commands: Commands,
    rsms: Populated<
        (NameOrEntity, &RsmMaterials, &GlobalTransform),
        (With<Mesh3d>, Without<MeshMaterial3d<RsmMaterial>>),
    >,
) {
    for (rsm, rsm_materials, global_transform) in rsms.into_inner() {
        let inverted_axis = global_transform.scale().is_negative_bitmask().count_ones();
        let material = if inverted_axis % 2 == 0 {
            rsm_materials.base.clone()
        } else {
            rsm_materials.inverted.clone()
        };
        commands.entity(rsm.entity).insert(MeshMaterial3d(material));
    }
}
