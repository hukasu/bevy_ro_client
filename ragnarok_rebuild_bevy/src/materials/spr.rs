use bevy::{
    asset::{load_internal_asset, Asset, Handle},
    pbr::{Material, MaterialPlugin},
    prelude::{Image, Shader},
    reflect::Reflect,
    render::render_resource::AsBindGroup,
};

const SPR_VERTEX_HANDLE: Handle<Shader> = Handle::weak_from_u128(u128::from_le_bytes([
    0x0e, 0xd8, 0xf3, 0xb1, 0x37, 0xfa, 0x41, 0xad, 0x96, 0x52, 0xe2, 0xd3, 0x2b, 0x3e, 0xee, 0xd5,
]));
const SPR_INDEXED_MATERIAL_HANDLE: Handle<Shader> = Handle::weak_from_u128(u128::from_le_bytes([
    0x8f, 0x19, 0x02, 0x50, 0xcf, 0x0c, 0x44, 0x4a, 0x92, 0x0c, 0xed, 0xe0, 0x79, 0x54, 0x58, 0x3f,
]));
const SPR_TRUE_COLOR_MATERIAL_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(u128::from_le_bytes([
        0xd3, 0x12, 0xa2, 0x35, 0xe7, 0x50, 0x42, 0xeb, 0x97, 0x55, 0x8c, 0x1d, 0xba, 0x23, 0xfd,
        0xa8,
    ]));

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Material Plugin
            .add_plugins(MaterialPlugin::<SprIndexedMaterial>::default())
            .add_plugins(MaterialPlugin::<SprTrueColorMaterial>::default());

        // Shader handles
        load_internal_asset!(
            app,
            SPR_VERTEX_HANDLE,
            "shaders/spr_vertex.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SPR_INDEXED_MATERIAL_HANDLE,
            "shaders/spr_indexed.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SPR_TRUE_COLOR_MATERIAL_HANDLE,
            "shaders/spr_true_color.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Clone, Asset, Reflect, AsBindGroup)]
pub struct SprIndexedMaterial {
    #[texture(0, sample_type = "u_int")]
    pub index_image: Handle<Image>,
    #[texture(1, dimension = "1d")]
    pub palette: Handle<Image>,
}

impl Material for SprIndexedMaterial {
    fn alpha_mode(&self) -> bevy::prelude::AlphaMode {
        bevy::prelude::AlphaMode::Blend
    }

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_VERTEX_HANDLE.into()
    }

    fn deferred_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_VERTEX_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_INDEXED_MATERIAL_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_INDEXED_MATERIAL_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor
            .vertex
            .shader_defs
            .push("SPR_INDEXED_PIPELINE".into());

        Ok(())
    }
}

#[derive(Clone, Asset, Reflect, AsBindGroup)]
pub struct SprTrueColorMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub color: Handle<Image>,
}

impl Material for SprTrueColorMaterial {
    fn alpha_mode(&self) -> bevy::prelude::AlphaMode {
        bevy::prelude::AlphaMode::Blend
    }

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_VERTEX_HANDLE.into()
    }

    fn deferred_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_VERTEX_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_TRUE_COLOR_MATERIAL_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_TRUE_COLOR_MATERIAL_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor
            .vertex
            .shader_defs
            .push("SPR_TRUE_COLOR_PIPELINE".into());

        Ok(())
    }
}
