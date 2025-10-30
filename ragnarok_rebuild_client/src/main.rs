mod client;
#[cfg(feature = "with-inspector")]
mod inspector_egui;

#[cfg(feature = "with-inspector")]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
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

use bevy_enhanced_input::EnhancedInputPlugin;
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
                #[expect(clippy::unwrap_used, reason = "Grf AssetReader's API needs rework")]
                let grf =
                    bevy_ragnarok_grf::AssetReader::new(std::path::Path::new("data.grf")).unwrap();
                Box::new(grf)
            }),
        );
    // Resources
    app.insert_resource(PointLightShadowMap { size: 16 });
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
                    format!("ragnarok_gat={log_level}"),
                    format!("ragnarok_gnd={log_level}"),
                    format!("ragnarok_grf={log_level}"),
                    format!("ragnarok_pal={log_level}"),
                    format!("ragnarok_rsm={log_level}"),
                    format!("ragnarok_rsw={log_level}"),
                    format!("ragnarok_spr={log_level}"),
                    format!("bevy_ragnarok_act={log_level}"),
                    format!("bevy_ragnarok_gat={log_level}"),
                    format!("bevy_ragnarok_gnd={log_level}"),
                    format!("bevy_ragnarok_grf={log_level}"),
                    format!("bevy_ragnarok_pal={log_level}"),
                    format!("bevy_ragnarok_rsm={log_level}"),
                    format!("bevy_ragnarok_rsw={log_level}"),
                    format!("bevy_ragnarok_spr={log_level}"),
                    "bevy_ragnarok_quad_tree=info".to_owned(),
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
    );
    app.add_plugins(EnhancedInputPlugin);
    app.add_plugins(RemotePlugin::default());
    app.add_plugins(RagnarokPlugin);

    #[cfg(feature = "with-inspector")]
    {
        app.add_plugins(FpsOverlayPlugin::default());
        app.add_plugins(inspector_egui::Plugin);
    }

    app.add_plugins(ClientPlugins);

    app.run();
}

struct ClientPlugins;

impl bevy::app::PluginGroup for ClientPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(client::plugin::Plugin)
            .add(bevy_ragnarok_act::plugin::Plugin {
                audio_path_prefix: paths::WAV_FILES_FOLDER.into(),
            })
            .add(bevy_ragnarok_gat::plugin::Plugin)
            .add(bevy_ragnarok_gnd::plugin::Plugin {
                texture_prefix: paths::TEXTURE_FILES_FOLDER.into(),
            })
            .add(bevy_ragnarok_pal::plugin::Plugin)
            .add(bevy_ragnarok_rsm::plugin::Plugin {
                texture_path_prefix: paths::TEXTURE_FILES_FOLDER.into(),
            })
            .add(bevy_ragnarok_rsw::plugin::Plugin {
                model_path_prefix: paths::MODEL_FILES_FOLDER.into(),
                ground_path_prefix: paths::GROUND_FILES_FOLDER.into(),
                altitude_path_prefix: paths::ALTITUDE_FILES_FOLDER.into(),
                sound_path_prefix: paths::WAV_FILES_FOLDER.into(),
            })
            .add(bevy_ragnarok_spr::plugin::Plugin)
            .add(bevy_ragnarok_camera::plugin::Plugin)
            .add(bevy_ragnarok_quad_tree::plugin::Plugin)
            .add(bevy_ragnarok_water_plane::plugin::Plugin {
                texture_prefix: paths::WATER_TEXTURE_FILES_FOLDER.into(),
            })
    }
}
