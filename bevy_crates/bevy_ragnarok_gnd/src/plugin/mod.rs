mod loader;

use std::borrow::Cow;

use bevy_app::AppExit;
use bevy_asset::{AssetApp, Assets, Handle, uuid_handle};
use bevy_log::error;
use bevy_math::{Quat, Vec2, Vec3, primitives::Plane3d};
use bevy_mesh::{Indices, Mesh, MeshBuilder, Meshable};

#[cfg(feature = "debug")]
use crate::debug;
use crate::{Cube, Ground, assets::GndAsset, material, plugin::loader::AssetLoader};

const GND_TOP_MESH: Handle<Mesh> = uuid_handle!("886618db-d316-482e-8aeb-c79a73e47f44");
const GND_EAST_MESH: Handle<Mesh> = uuid_handle!("8ddb2470-39cd-4083-b37d-93d2a84bb2d6");
const GND_NORTH_MESH: Handle<Mesh> = uuid_handle!("f1bcac29-8498-4f61-8f7c-b07b8112a61b");

pub struct Plugin {
    pub texture_prefix: Cow<'static, str>,
}

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Asset Loader
        app.init_asset::<GndAsset>()
            .register_asset_loader(AssetLoader {
                texture_prefix: self.texture_prefix.clone(),
            });

        // Material
        app.add_plugins(material::Plugin);

        // Types
        app.register_type::<Ground>();
        app.register_type::<Cube>();

        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);
    }

    fn finish(&self, app: &mut bevy_app::App) {
        let Some(mut meshes) = app.world_mut().get_resource_mut::<Assets<Mesh>>() else {
            error!("Assets<Mesh> does not exits.");
            app.world_mut().write_message(AppExit::from_code(1));
            return;
        };

        if let Err(err) = meshes.insert(
            GND_TOP_MESH.id(),
            Plane3d::new(Vec3::NEG_Y, Vec2::splat(0.5))
                .mesh()
                .build()
                .with_inserted_indices(Indices::U16(vec![0, 3, 1, 0, 2, 3])),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
            return;
        }

        if let Err(err) = meshes.insert(
            GND_EAST_MESH.id(),
            Plane3d::new(Vec3::X, Vec2::splat(0.5))
                .mesh()
                .build()
                .translated_by(Vec3::new(0.5, 0., 0.)),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
            return;
        }

        if let Err(err) = meshes.insert(
            GND_NORTH_MESH.id(),
            Plane3d::new(Vec3::NEG_Z, Vec2::splat(0.5))
                .mesh()
                .build()
                .rotated_by(Quat::from_xyzw(0.0, 1., 0.0, 0.0))
                .translated_by(Vec3::new(0., 0., 0.5)),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
        }
    }
}
