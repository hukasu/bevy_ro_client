use bevy::{
    app::{App, PluginGroup, PostStartup, Startup},
    asset::{AssetServer, Handle},
    core::Name,
    ecs::system::{Commands, Res},
    log::LogPlugin,
    math::{Quat, Vec3},
    prelude::SpatialBundle,
    render::texture::{ImagePlugin, ImageSamplerDescriptor},
    scene::Scene,
    transform::components::Transform,
    DefaultPlugins,
};
#[cfg(feature = "with-inspector")]
use bevy::{
    audio::SpatialListener,
    ecs::{entity::Entity, query::With, system::Query},
};
#[cfg(not(feature = "with-inspector"))]
use bevy::{core_pipeline::core_3d::Camera3dBundle, math::Vec3};
#[cfg(feature = "with-inspector")]
use bevy_flycam::FlyCam;
#[cfg(feature = "with-inspector")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use ragnarok_rebuild_bevy::{assets::rsw, RagnarokPlugin};

fn main() {
    let log_level = "debug";

    let mut app = App::new();
    app
    // Plugins
    .add_plugins(RagnarokPlugin)
        .add_plugins(
            DefaultPlugins
                .build()
                .disable::<bevy::asset::AssetPlugin>()
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: format!("wgpu=error,naga=warn,ragnarok_rebuild_client={log_level},ragnarok_rebuild_bevy={log_level},ragnarok_rebuild_common={log_level}"),
                    custom_layer: |_| None
                }).set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor::nearest()
                }),
        )
        .add_systems(Startup, load_map);

    #[cfg(not(feature = "with-inspector"))]
    {
        app.add_systems(Startup, spawn_camera);
    }
    #[cfg(feature = "with-inspector")]
    {
        app.add_plugins(WorldInspectorPlugin::default())
            .add_plugins(bevy_flycam::prelude::PlayerPlugin)
            .insert_resource(bevy_flycam::prelude::MovementSettings {
                sensitivity: 0.00015, // default: 0.00012
                speed: 24.0,          // default: 12.0
            })
            .add_systems(PostStartup, add_listener_to_fly_cam);
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

#[cfg(feature = "with-inspector")]
fn add_listener_to_fly_cam(mut commands: Commands, flycams: Query<Entity, With<FlyCam>>) {
    let Ok(flycam) = flycams.get_single() else {
        bevy::log::error!("Zero or more than one FlyCam present.");
        return;
    };

    commands.entity(flycam).insert(SpatialListener::default());
}

fn load_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    bevy::log::trace!("Loading map");
    let rsw_handle: Handle<Scene> = asset_server.load("data/prontera.rsw");

    commands.spawn((
        Name::new("World"),
        rsw::World,
        rsw_handle,
        SpatialBundle {
            transform: Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::PI))
                .with_scale(Vec3::splat(0.2)),
            ..Default::default()
        },
    ));
}
