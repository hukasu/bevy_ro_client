use bevy_asset::AssetApp;
use bevy_camera::{
    primitives::Aabb,
    visibility::{InheritedVisibility, ViewVisibility, Visibility},
};
use bevy_ecs::{
    hierarchy::{ChildOf, Children},
    name::Name,
};
use bevy_transform::components::{GlobalTransform, Transform, TransformTreeChanged};

use crate::{Tile, assets::Gat, loader::AssetLoader};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Assets
        app.init_asset::<Gat>().register_asset_loader(AssetLoader);
        // Register types
        app.register_type::<Tile>();

        // Things necessary for the Scene
        app.register_type::<Name>();
        app.register_type::<Aabb>();
        app.register_type::<ChildOf>();
        app.register_type::<Children>();
        app.register_type::<Transform>();
        app.register_type::<GlobalTransform>();
        app.register_type::<TransformTreeChanged>();
        app.register_type::<Visibility>();
        app.register_type::<InheritedVisibility>();
        app.register_type::<ViewVisibility>();

        #[cfg(feature = "debug")]
        {
            app.add_plugins(crate::debug::Plugin);
        }
    }
}
