use bevy::{
    asset::{load_internal_asset, Asset, AssetApp, Handle},
    pbr::{Material, MaterialPlugin},
    prelude::{AlphaMode, Image, Shader},
    reflect::Reflect,
    render::render_resource::AsBindGroup,
};

const GND_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xb7fa811ae840469eb97291bb81c55dfd);

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Asset
            .init_asset::<GndMaterial>()
            .register_asset_reflect::<GndMaterial>()
            // Material Plugin
            .add_plugins(MaterialPlugin::<GndMaterial>::default());

        // Shader handles
        load_internal_asset!(
            app,
            GND_SHADER_HANDLE,
            "shaders/gnd_shader.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Clone, Asset, Reflect, AsBindGroup)]
pub struct GndMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub color_texture: Handle<Image>,
}

impl Material for GndMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn deferred_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        GND_SHADER_HANDLE.into()
    }
}
