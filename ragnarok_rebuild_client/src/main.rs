mod client;
#[cfg(feature = "with-inspector")]
mod inspector_egui;
#[cfg(not(feature = "with-inspector"))]
mod other {
    use bevy::{
        ecs::system::Commands,
        prelude::{App, IntoScheduleConfigs, Startup},
    };
    use ragnarok_rebuild_bevy::tables;
    use ragnarok_rsw::events::LoadWorld;

    pub struct Plugin;

    impl bevy::app::Plugin for Plugin {
        fn build(&self, app: &mut App) {
            app.add_systems(
                Startup,
                spawn_camera.after(tables::system_sets::TableSystems::TableStartup),
            );
        }
    }

    fn spawn_camera(mut commands: Commands) {
        commands.trigger(LoadWorld {
            world: "prontera.rsw".into(),
        });
    }
}

use bevy::{
    app::{App, PluginGroup, PluginGroupBuilder},
    asset::{io::AssetSourceBuilder, AssetApp, AssetPlugin},
    image::ImageSamplerDescriptor,
    light::PointLightShadowMap,
    log::LogPlugin,
    prelude::ImagePlugin,
    remote::RemotePlugin,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};

use ragnarok_rebuild_bevy::{
    assets::paths::{self, BASE_DATA_FOLDER},
    RagnarokPlugin,
};

fn main() {
    let log_level = "trace";

    let mut app = App::new();
    app
        // Asset Sources
        .register_asset_source("bgm", AssetSourceBuilder::platform_default("BGM/", None))
        .register_asset_source(
            "system",
            AssetSourceBuilder::platform_default("System/", None),
        )
        .register_asset_source(
            bevy::asset::io::AssetSourceId::Default,
            bevy::asset::io::AssetSourceBuilder::default().with_reader(|| {
                let grf = ragnarok_grf::reader::AssetReader::new(std::path::Path::new("data.grf"))
                    .unwrap();
                Box::new(grf)
            }),
        );
    // Plugins
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                file_path: BASE_DATA_FOLDER.to_owned(),
                ..Default::default()
            })
            .set(LogPlugin {
                level: bevy::log::Level::INFO,
                filter: [
                    "wgpu=error".to_owned(),
                    "naga=warn".to_owned(),
                    format!("ragnarok_rebuild_client={log_level}"),
                    format!("ragnarok_rebuild_bevy={log_level}"),
                    format!("ragnarok_rebuild_assets={log_level}"),
                    format!("ragnarok_rebuild_common={log_level}"),
                    format!("ragnarok_act={log_level}"),
                    format!("ragnarok_grf={log_level}"),
                    format!("ragnarok_pal={log_level}"),
                    format!("ragnarok_rsm={log_level}"),
                    format!("ragnarok_rsw={log_level}"),
                    format!("ragnarok_spr={log_level}"),
                ]
                .join(","),
                custom_layer: |_| None,
                ..Default::default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor::nearest(),
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Ragnarok Rebuild".into(),
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
    )
    .add_plugins(RemotePlugin::default())
    .insert_resource(PointLightShadowMap { size: 16 });
    app.add_plugins((
        RagnarokPlugin,
        bevy_ragnarok_act::plugin::Plugin {
            audio_path_prefix: paths::WAV_FILES_FOLDER.into(),
        },
        bevy_ragnarok_pal::plugin::Plugin,
        ragnarok_rsm::plugin::Plugin {
            texture_path_prefix: paths::TEXTURE_FILES_FOLDER.into(),
        },
        ragnarok_rsw::plugin::Plugin {
            model_path_prefix: paths::MODEL_FILES_FOLDER.into(),
            ground_path_prefix: paths::GROUND_FILES_FOLDER.into(),
            altitude_path_prefix: paths::ALTITUDE_FILES_FOLDER.into(),
            sound_path_prefix: paths::WAV_FILES_FOLDER.into(),
        },
        bevy_ragnarok_spr::plugin::Plugin,
    ));

    #[cfg(not(feature = "with-inspector"))]
    {
        app.add_plugins(other::Plugin);
    }
    #[cfg(feature = "with-inspector")]
    {
        app.add_plugins(inspector_egui::Plugin);
    }

    app.add_plugins(ClientPlugins);

    app.run();
}

struct ClientPlugins;

impl bevy::app::PluginGroup for ClientPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(client::Plugin)
    }
}
