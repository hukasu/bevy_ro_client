use bevy::{
    app::{App, PluginGroup, PostStartup, Update},
    asset::{io::AssetSourceBuilder, AssetApp},
    ecs::system::Commands,
    input::ButtonInput,
    log::LogPlugin,
    pbr::{DirectionalLightShadowMap, PointLightShadowMap},
    prelude::{not, IntoSystemConfigs, KeyCode, Res},
    render::texture::{ImagePlugin, ImageSamplerDescriptor},
    window::{PrimaryWindow, Window},
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

use ragnarok_rebuild_bevy::{
    world::{LoadWorld, UnloadWorld},
    RagnarokPlugin,
};

fn main() {
    let log_level = "trace";

    let mut app = App::new();
    app
        // Resources
        .insert_resource(DirectionalLightShadowMap {size: 2048})
        .insert_resource(PointLightShadowMap { size: 32})
        // Asset Sources
        .register_asset_source("bgm", AssetSourceBuilder::platform_default("BGM/", None))
        // Plugins
        .add_plugins(RagnarokPlugin)
        .add_plugins(
            DefaultPlugins
                .build()
                // AssetPlugin is initialized by RagnarokPlugin
                .disable::<bevy::asset::AssetPlugin>()
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: format!("wgpu=error,naga=warn,ragnarok_rebuild_client={log_level},ragnarok_rebuild_bevy={log_level},ragnarok_rebuild_assets={log_level},ragnarok_rebuild_common={log_level}"),
                    custom_layer: |_| None
                }).set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor::nearest()
                }),
        );

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

    app
        // Systems
        .add_systems(Update, load_map.run_if(not(is_input_captured)));

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

fn is_input_captured(windows: Query<&Window, With<PrimaryWindow>>) -> bool {
    if cfg!(feature = "with-inspector") {
        if let Ok(primary_window) = windows.get_single() {
            matches!(
                primary_window.cursor.grab_mode,
                bevy::window::CursorGrabMode::Confined
            )
        } else {
            false
        }
    } else {
        false
    }
}

fn load_map(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        commands.trigger(UnloadWorld);
        commands.trigger(LoadWorld {
            world: "data/prontera.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyW) {
        commands.trigger(UnloadWorld);
        commands.trigger(LoadWorld {
            world: "data/morocc.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyE) {
        commands.trigger(UnloadWorld);
        commands.trigger(LoadWorld {
            world: "data/payon.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.trigger(UnloadWorld);
        commands.trigger(LoadWorld {
            world: "data/yuno.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyA) {
        commands.trigger(UnloadWorld);
        commands.trigger(LoadWorld {
            world: "data/pay_dun01.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyS) {
        commands.trigger(UnloadWorld);
        commands.trigger(LoadWorld {
            world: "data/moc_pryd03.rsw".into(),
        });
    }
}
