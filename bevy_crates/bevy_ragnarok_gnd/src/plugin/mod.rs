mod loader;

use std::borrow::Cow;

use bevy_app::AppExit;
use bevy_asset::{AssetApp, Assets, Handle, RenderAssetUsages, uuid_handle};
use bevy_log::error;
use bevy_math::{Vec2, Vec3};
use bevy_mesh::{Indices, Mesh};

#[cfg(feature = "debug")]
use crate::debug;
use crate::{Cube, Ground, assets::GndAsset, material, plugin::loader::AssetLoader};

const GND_TOP_MESH: Handle<Mesh> = uuid_handle!("886618db-d316-482e-8aeb-c79a73e47f44");
const GND_EAST_MESH: Handle<Mesh> = uuid_handle!("8ddb2470-39cd-4083-b37d-93d2a84bb2d6");
const GND_NORTH_MESH: Handle<Mesh> = uuid_handle!("f1bcac29-8498-4f61-8f7c-b07b8112a61b");
const INDICES: [u16; 6] = [0, 1, 2, 1, 3, 2];

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

        let asset_usage = if cfg!(feature = "debug") {
            RenderAssetUsages::all()
        } else {
            RenderAssetUsages::RENDER_WORLD
        };

        if let Err(err) = meshes.insert(
            GND_TOP_MESH.id(),
            Mesh::new(bevy_mesh::PrimitiveTopology::TriangleList, asset_usage)
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        Vec3::new(-0.5, 0.5, -0.5),
                        Vec3::new(0.5, 0.5, -0.5),
                        Vec3::new(-0.5, 0.5, 0.5),
                        Vec3::new(0.5, 0.5, 0.5),
                    ],
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::NEG_Y; 4])
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; 4])
                .with_inserted_indices(Indices::U16(INDICES.to_vec())),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
            return;
        }

        if let Err(err) = meshes.insert(
            GND_EAST_MESH.id(),
            Mesh::new(bevy_mesh::PrimitiveTopology::TriangleList, asset_usage)
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        Vec3::new(0.5, 0.5, 0.5),
                        Vec3::new(0.5, 0.5, -0.5),
                        Vec3::new(0.5, -0.5, 0.5),
                        Vec3::new(0.5, -0.5, -0.5),
                    ],
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::X; 4])
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; 4])
                .with_inserted_indices(Indices::U16(INDICES.to_vec())),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
            return;
        }

        if let Err(err) = meshes.insert(
            GND_NORTH_MESH.id(),
            Mesh::new(bevy_mesh::PrimitiveTopology::TriangleList, asset_usage)
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    vec![
                        Vec3::new(-0.5, 0.5, 0.5),
                        Vec3::new(0.5, 0.5, 0.5),
                        Vec3::new(-0.5, -0.5, 0.5),
                        Vec3::new(0.5, -0.5, 0.5),
                    ],
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![Vec3::NEG_Z; 4])
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; 4])
                .with_inserted_indices(Indices::U16(INDICES.to_vec())),
        ) {
            error!("{err}");
            app.world_mut().write_message(AppExit::from_code(1));
        }
    }
}
