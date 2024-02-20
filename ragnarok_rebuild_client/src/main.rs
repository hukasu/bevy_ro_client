use bevy::{
    app::{App, PluginGroup, Startup},
    asset::{AssetServer, Handle},
    core::Name,
    core_pipeline::clear_color::ClearColor,
    ecs::system::{Commands, Res},
    log::LogPlugin,
    math::Quat,
    prelude::SpatialBundle,
    render::color::Color,
    transform::components::Transform,
    DefaultPlugins,
};
#[cfg(not(feature = "with-inspector"))]
use bevy::{core_pipeline::core_3d::Camera3dBundle, math::Vec3};

use ragnarok_rebuild_bevy::{assets::rsw, world, RagnarokPlugin};

fn main() {
    let log_level = "debug";

    let mut app = App::new();
    app
    // Resources
    .insert_resource(ClearColor(Color::ALICE_BLUE))
    // Plugins
    .add_plugins(RagnarokPlugin)
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::asset::AssetPlugin>()
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: format!("wgpu=error,naga=warn,ragnarok_rebuild_client={log_level},ragnarok_rebuild_bevy={log_level},ragnarok_rebuild_common={log_level}"),
                }),
        )
        .add_systems(Startup, load_map);

    #[cfg(not(feature = "with-inspector"))]
    {
        app.add_systems(Startup, spawn_camera);
    }
    #[cfg(feature = "with-inspector")]
    {
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
            .add_plugins(bevy_flycam::prelude::PlayerPlugin)
            .insert_resource(bevy_flycam::prelude::MovementSettings {
                sensitivity: 0.00015, // default: 0.00012
                speed: 120.0,         // default: 12.0
            });
    }

    app.run();
}

#[cfg(not(feature = "with-inspector"))]
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 2500., 30.)
            .looking_at(Vec3::new(0., 0., 0.), Vec3::NEG_Z),
        ..Default::default()
    });
}

fn load_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    bevy::log::trace!("Loading map");
    let rsw_handle: Handle<rsw::Asset> = asset_server.load("data/prontera.rsw");
    // let rsw_handle: Handle<rsw::Asset> = asset_server.load("data/payon.rsw");
    // let rsw_handle: Handle<rsw::Asset> = asset_server.load("data/pay_dun00.rsw");
    // let rsw_handle: Handle<rsw::Asset> = asset_server.load("data/moc_pryd03.rsw");
    // V2_5
    // let rsw_handle: Handle<rsw::Asset> = asset_server.load("data/1@4igd.rsw");

    commands.spawn((
        world::World,
        rsw_handle,
        SpatialBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
            ..Default::default()
        },
        Name::new("prontera"),
    ));
}
