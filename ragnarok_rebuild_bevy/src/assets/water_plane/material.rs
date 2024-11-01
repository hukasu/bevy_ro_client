use bevy::{
    asset::{load_internal_asset, Asset, AssetApp, Handle},
    pbr::{Material, MaterialPlugin},
    prelude::{AlphaMode, Image, Shader},
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderType},
};

const WATER_PLANE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x13c76198ee094c50bbe45609d880269e);

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Asset
            .init_asset::<WaterPlaneMaterial>()
            .register_asset_reflect::<WaterPlaneMaterial>()
            // Material Plugin
            .add_plugins(MaterialPlugin::<WaterPlaneMaterial>::default());

        // Shader handles
        load_internal_asset!(
            app,
            WATER_PLANE_SHADER_HANDLE,
            "shaders/water_plane_shader.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Debug, Clone, Asset, Reflect, AsBindGroup)]
pub struct WaterPlaneMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub wave: Wave,
}

impl Material for WaterPlaneMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn deferred_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }
}

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct Wave {
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_pitch: f32,
}
