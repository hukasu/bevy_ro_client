use bevy_asset::{Asset, AssetApp, Handle, embedded_asset};
use bevy_color::LinearRgba;
use bevy_image::Image;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy_reflect::Reflect;
use bevy_render::{
    alpha::AlphaMode,
    render_resource::{
        AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    },
};
use bevy_shader::ShaderRef;

pub(crate) struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Asset
            .init_asset::<SprIndexedMaterial>()
            .register_asset_reflect::<SprIndexedMaterial>()
            .init_asset::<SprTrueColorMaterial>()
            .register_asset_reflect::<SprTrueColorMaterial>()
            // Material Plugin
            .add_plugins(MaterialPlugin::<SprIndexedMaterial>::default())
            .add_plugins(MaterialPlugin::<SprTrueColorMaterial>::default());

        // Shader handles
        embedded_asset!(app, "shaders/spr_vertex.wgsl");
        embedded_asset!(app, "shaders/spr_fragment.wgsl");
        embedded_asset!(app, "shaders/spr_prepass_fragment.wgsl");
    }
}

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct SprUniform {
    pub uv_flip: u32,
    pub tint: LinearRgba,
}

#[derive(Clone, Asset, Reflect, AsBindGroup)]
pub struct SprIndexedMaterial {
    #[uniform(0)]
    pub uniform: SprUniform,
    #[texture(1, sample_type = "u_int")]
    pub index_image: Handle<Image>,
    #[texture(2, dimension = "1d")]
    pub palette: Handle<Image>,
}

impl Material for SprIndexedMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/material/shaders/spr_vertex.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/material/shaders/spr_vertex.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/material/shaders/spr_fragment.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/material/shaders/spr_prepass_fragment.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor
            .vertex
            .shader_defs
            .push("SPR_INDEXED_PIPELINE".into());

        if let Some(frag_descriptor) = &mut descriptor.fragment {
            frag_descriptor
                .shader_defs
                .push("SPR_INDEXED_PIPELINE".into());
        }

        Ok(())
    }
}

#[derive(Clone, Asset, Reflect, AsBindGroup)]
pub struct SprTrueColorMaterial {
    #[uniform(0)]
    pub uniform: SprUniform,
    #[texture(1)]
    #[sampler(2)]
    pub color: Handle<Image>,
}

impl Material for SprTrueColorMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/assets/shaders/spr_vertex.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/assets/shaders/spr_vertex.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/assets/shaders/spr_fragment.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_spr/assets/shaders/spr_prepass_fragment.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor
            .vertex
            .shader_defs
            .push("SPR_TRUE_COLOR_PIPELINE".into());

        if let Some(frag_descriptor) = &mut descriptor.fragment {
            frag_descriptor
                .shader_defs
                .push("SPR_TRUE_COLOR_PIPELINE".into());
        }

        Ok(())
    }
}
