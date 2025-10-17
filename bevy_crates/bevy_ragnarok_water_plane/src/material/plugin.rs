use bevy_app::App;
use bevy_asset::{AssetApp, load_internal_asset};
use bevy_pbr::MaterialPlugin;
use bevy_shader::Shader;

use crate::material::{WATER_PLANE_SHADER_HANDLE, WaterPlaneMaterial};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        // Asset
        app.init_asset::<WaterPlaneMaterial>()
            .register_asset_reflect::<WaterPlaneMaterial>();
        // Material Plugin
        app.add_plugins(MaterialPlugin::<WaterPlaneMaterial>::default());

        // Shader handles
        load_internal_asset!(
            app,
            WATER_PLANE_SHADER_HANDLE,
            "shaders/water_plane_shader.wgsl",
            Shader::from_wgsl
        );
    }
}
