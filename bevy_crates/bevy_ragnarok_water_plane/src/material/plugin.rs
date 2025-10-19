use bevy_app::App;
use bevy_asset::{AssetApp, embedded_asset};
use bevy_pbr::MaterialPlugin;

use crate::material::WaterPlaneMaterial;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // Asset
        app.init_asset::<WaterPlaneMaterial>()
            .register_asset_reflect::<WaterPlaneMaterial>();
        // Material Plugin
        app.add_plugins(MaterialPlugin::<WaterPlaneMaterial>::default());

        // Shader handles
        embedded_asset!(app, "shaders/water_plane_shader.wgsl");
    }
}
