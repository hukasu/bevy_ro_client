mod loader;

use std::{borrow::Cow, f32::consts::PI};

use bevy_app::AppExit;
use bevy_asset::{AssetApp, Assets, Handle, uuid_handle};
use bevy_log::error;
use bevy_math::{Quat, Vec2, Vec3, primitives::Plane3d};
use bevy_mesh::{Mesh, MeshBuilder, Meshable};

#[cfg(feature = "debug")]
use crate::debug;
use crate::{Ground, assets::GndAsset, material, plugin::loader::AssetLoader};

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
            Plane3d::new(Vec3::NEG_Y, Vec2::splat(0.5)).mesh().build(),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
            return;
        }

        if let Err(err) = meshes.insert(
            GND_EAST_MESH.id(),
            Plane3d::new(Vec3::NEG_X, Vec2::splat(0.5))
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
            Plane3d::new(Vec3::Z, Vec2::splat(0.5))
                .mesh()
                .build()
                .rotated_by(Quat::from_rotation_y(PI))
                .translated_by(Vec3::new(0., 0., 0.5)),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
        }
    }
}
