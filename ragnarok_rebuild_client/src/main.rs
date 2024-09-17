mod client;
#[cfg(feature = "with-inspector")]
mod inspector_egui;

use bevy::{
    app::{App, PluginGroup, Update},
    asset::{io::AssetSourceBuilder, AssetApp, AssetPlugin},
    ecs::system::Commands,
    input::ButtonInput,
    log::LogPlugin,
    prelude::{not, IntoSystemConfigs, KeyCode, Query, Res, With},
    render::texture::{ImagePlugin, ImageSamplerDescriptor},
    window::{PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};

#[cfg(not(feature = "with-inspector"))]
use bevy::{
    core_pipeline::core_3d::Camera3dBundle, math::Vec3, prelude::Startup, prelude::Transform,
};

use ragnarok_rebuild_bevy::{
    assets::{grf, paths::BASE_DATA_FOLDER, rsw::LoadWorld},
    RagnarokPlugin,
};

use self::client::ClientPlugin;

fn main() {
    let log_level = "trace";

    let mut app = App::new();
    app
        // Asset Sources
        .register_asset_source("bgm", AssetSourceBuilder::platform_default("BGM/", None))
        .register_asset_source("system", AssetSourceBuilder::platform_default("System/", None))
        .register_asset_source(
            bevy::asset::io::AssetSourceId::Default,
            bevy::asset::io::AssetSourceBuilder::default().with_reader(|| {
                let grf = grf::AssetReader::new(std::path::Path::new("data.grf")).unwrap();
                Box::new(grf)
            }),
        )
        // Plugins
        .add_plugins(
            DefaultPlugins
                .build()
                .set(AssetPlugin {
                    file_path: BASE_DATA_FOLDER.to_owned(),
                    ..Default::default()
                })
                .set(LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: format!("wgpu=error,naga=warn,ragnarok_rebuild_client={log_level},ragnarok_rebuild_bevy={log_level},ragnarok_rebuild_assets={log_level},ragnarok_rebuild_common={log_level}"),
                    custom_layer: |_| None
                })
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor::nearest()
                }).set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Ragnarok Rebuild".into(),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
        )
        .add_plugins(RagnarokPlugin);

    #[cfg(not(feature = "with-inspector"))]
    {
        app.add_systems(Startup, spawn_camera);
    }
    #[cfg(feature = "with-inspector")]
    {
        app.add_plugins(inspector_egui::Plugin);
    }

    app
        // Plugins
        .add_plugins(ClientPlugin)
        // Systems
        .add_systems(Update, load_map.run_if(not(is_input_captured)));

    app.run();
}

#[cfg(not(feature = "with-inspector"))]
fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., 500., 30.)
            .looking_at(Vec3::new(0., 0., 0.), Vec3::NEG_Z),
        ..Default::default()
    });
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
        commands.trigger(LoadWorld {
            world: "prontera.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyW) {
        commands.trigger(LoadWorld {
            world: "morocc.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyE) {
        commands.trigger(LoadWorld {
            world: "payon.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyR) {
        commands.trigger(LoadWorld {
            world: "yuno.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyA) {
        commands.trigger(LoadWorld {
            world: "pay_dun00.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyS) {
        commands.trigger(LoadWorld {
            world: "moc_pryd03.rsw".into(),
        });
    } else if keyboard_input.just_pressed(KeyCode::KeyD) {
        commands.trigger(LoadWorld {
            world: "prt_in.rsw".into(),
        });
    }
}
