pub(crate) mod plugin;

use bevy_asset::{Asset, AssetPath, Handle, embedded_path};
use bevy_color::LinearRgba;
use bevy_image::Image;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy_reflect::Reflect;
use bevy_render::{
    alpha::AlphaMode,
    render_resource::{
        AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    },
    storage::ShaderStorageBuffer,
};
use bevy_shader::ShaderRef;

#[derive(Clone, Asset, Reflect, AsBindGroup)]
pub struct SprMaterial {
    #[texture(0, sample_type = "u_int")]
    pub index_image: Handle<Image>,
    #[texture(1, dimension = "1d")]
    pub palette: Handle<Image>,
    #[texture(2)]
    #[sampler(3)]
    pub true_color_image: Handle<Image>,
    #[storage(4, read_only)]
    pub indexed_uvs: Handle<ShaderStorageBuffer>,
    #[storage(5, read_only)]
    pub true_color_uvs: Handle<ShaderStorageBuffer>,
    #[storage(6, read_only)]
    pub uv_index: Handle<ShaderStorageBuffer>,
    #[storage(7, read_only)]
    pub tint: Handle<ShaderStorageBuffer>,
    #[storage(8, read_only)]
    pub flipped_uvs: Handle<ShaderStorageBuffer>,
}

impl SprMaterial {
    /// Stride of the items on [`SprMaterial::indexed_uvs`]
    /// and [`SprMaterial::true_color_uvs`].
    pub const UV_STRIDE: usize = 4 * 4;
    /// Stride of the items on [`SprMaterial::uv_index`].
    pub const UV_INDEX_STRIDE: usize = 4;
    /// Stride of the items on [`SprMaterial::tint`].
    pub const TINT_STRIDE: usize = 4 * 4;
    /// Stride of the items on [`SprMaterial::flipped_uvs`].
    pub const FLIPPED_UVS_STRIDE: usize = 4;
}

impl Material for SprMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }

    fn vertex_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/spr_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/spr_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/spr_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/spr_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let label = descriptor.label.get_or_insert("shader".into());
        *label = format!("spr_material_{label}").into();

        Ok(())
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
    #[storage(3, read_only)]
    pub uvs: Handle<ShaderStorageBuffer>,
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
    #[storage(3, read_only)]
    pub uvs: Handle<ShaderStorageBuffer>,
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
