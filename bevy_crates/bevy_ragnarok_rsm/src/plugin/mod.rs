mod loader;

use bevy_animation::{AnimationPlayer, AnimationTarget, graph::AnimationGraphHandle};
use bevy_app::PostUpdate;
use bevy_asset::AssetApp;
use bevy_camera::visibility::{InheritedVisibility, ViewVisibility, Visibility};
use bevy_ecs::{
    hierarchy::{ChildOf, Children},
    name::{Name, NameOrEntity},
    query::{With, Without},
    schedule::IntoScheduleConfigs,
    system::{Commands, Populated},
};
use bevy_mesh::Mesh3d;
use bevy_pbr::MeshMaterial3d;
use bevy_transform::{
    TransformSystems,
    components::{GlobalTransform, Transform, TransformTreeChanged},
};
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

        // Types needed for Scene
        app.register_type::<Name>();
        app.register_type::<Children>();
        app.register_type::<ChildOf>();
        app.register_type::<Transform>();
        app.register_type::<GlobalTransform>();
        app.register_type::<TransformTreeChanged>();
        app.register_type::<Visibility>();
        app.register_type::<InheritedVisibility>();
        app.register_type::<ViewVisibility>();
        app.register_type::<AnimationPlayer>();
        app.register_type::<AnimationGraphHandle>();
        app.register_type::<AnimationTarget>();
        app.register_type::<Mesh3d>();

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
