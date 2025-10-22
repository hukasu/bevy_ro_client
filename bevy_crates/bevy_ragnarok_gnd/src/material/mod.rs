use bevy_asset::{Asset, AssetApp, Handle, load_internal_asset, uuid_handle};
use bevy_image::Image;
use bevy_math::Vec2;
use bevy_pbr::{Material, MaterialPlugin};
use bevy_reflect::Reflect;
use bevy_render::{
    alpha::AlphaMode,
    render_asset::RenderAssets,
    render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderType},
    texture::GpuImage,
};
use bevy_shader::{Shader, ShaderRef};

const GND_SHADER_HANDLE: Handle<Shader> = uuid_handle!("b7fa811a-e840-469e-b972-91bb81c55dfd");

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
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
#[data(0, GndCubeFace, binding_array(10))]
#[data(1, GndSurface, binding_array(11))]
#[bindless(index_table(range(0..4)))]
pub struct GndMaterial {
    pub bottom_left: f32,
    pub bottom_right: f32,
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left_uv: Vec2,
    pub bottom_right_uv: Vec2,
    pub top_left_uv: Vec2,
    pub top_right_uv: Vec2,
    #[texture(2)]
    #[sampler(3)]
    pub texture: Handle<Image>,
}

impl Material for GndMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn prepass_vertex_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn vertex_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy_pbr::MaterialPipeline,
        descriptor: &mut bevy_render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy_mesh::MeshVertexBufferLayoutRef,
        _key: bevy_pbr::MaterialPipelineKey<Self>,
    ) -> bevy_ecs::error::Result<(), bevy_render::render_resource::SpecializedMeshPipelineError>
    {
        descriptor.label = Some(
            format!(
                "gnd_{}",
                descriptor
                    .label
                    .as_ref()
                    .map(|label| label.as_ref())
                    .unwrap_or("shader")
            )
            .into(),
        );
        Ok(())
    }
}

#[derive(Debug, ShaderType)]
pub struct GndCubeFace {
    pub bottom_left: f32,
    pub bottom_right: f32,
    pub top_left: f32,
    pub top_right: f32,
}

impl AsBindGroupShaderType<GndCubeFace> for GndMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> GndCubeFace {
        GndCubeFace {
            bottom_left: self.bottom_left,
            bottom_right: self.bottom_right,
            top_left: self.top_left,
            top_right: self.top_right,
        }
    }
}

#[derive(Debug, ShaderType)]
pub struct GndSurface {
    pub bottom_left_uv: Vec2,
    pub bottom_right_uv: Vec2,
    pub top_left_uv: Vec2,
    pub top_right_uv: Vec2,
}

impl AsBindGroupShaderType<GndSurface> for GndMaterial {
    fn as_bind_group_shader_type(&self, _images: &RenderAssets<GpuImage>) -> GndSurface {
        GndSurface {
            bottom_left_uv: self.bottom_left_uv,
            bottom_right_uv: self.bottom_right_uv,
            top_left_uv: self.top_left_uv,
            top_right_uv: self.top_right_uv,
        }
    }
}
