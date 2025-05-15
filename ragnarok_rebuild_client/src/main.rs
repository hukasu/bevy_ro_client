mod client;
#[cfg(feature = "with-inspector")]
mod inspector_egui;
mod states;
#[cfg(not(feature = "with-inspector"))]
mod other {
    use bevy::{
        ecs::system::Commands,
        math::Vec3,
        prelude::{App, Camera3d, IntoScheduleConfigs, Startup, Transform},
    };
    use ragnarok_rebuild_bevy::{assets::rsw::LoadWorld, tables};

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
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(0., 500., 30.).looking_at(Vec3::new(0., 0., 0.), Vec3::NEG_Z),
        ));

        commands.trigger(LoadWorld {
            world: "prontera.rsw".into(),
        });
    }
}

use bevy::{
    app::{App, PluginGroup, PluginGroupBuilder},
    asset::{io::AssetSourceBuilder, AssetApp, AssetPlugin},
    image::ImageSamplerDescriptor,
    log::LogPlugin,
    pbr::PointLightShadowMap,
    prelude::ImagePlugin,
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
                    filter: format!("wgpu=error,naga=warn,ragnarok_rebuild_client={log_level},ragnarok_rebuild_bevy={log_level},ragnarok_rebuild_assets={log_level},ragnarok_rebuild_common={log_level},ragnarok_act={log_level},ragnarok_rsm={log_level}"),
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
                })
        ).insert_resource(PointLightShadowMap {size: 16});
    app.add_plugins((
        RagnarokPlugin,
        ragnarok_act::plugin::Plugin {
            audio_path_prefix: paths::WAV_FILES_FOLDER.into(),
        },
        ragnarok_pal::plugin::Plugin,
        ragnarok_rsm::plugin::Plugin {
            texture_path_prefix: paths::TEXTURE_FILES_FOLDER.into(),
        },
        ragnarok_spr::plugin::Plugin,
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
        PluginGroupBuilder::start::<Self>()
            .add(client::ClientPlugin)
            .add(states::Plugin)
    }
}
