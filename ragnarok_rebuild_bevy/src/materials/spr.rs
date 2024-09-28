use bevy::{
    asset::{load_internal_asset, Asset, AssetApp, Handle},
    pbr::{Material, MaterialPlugin},
    prelude::{Image, Shader},
    reflect::Reflect,
    render::render_resource::AsBindGroup,
};

const SPR_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(u128::from_le_bytes([
    0x0e, 0xd8, 0xf3, 0xb1, 0x37, 0xfa, 0x41, 0xad, 0x96, 0x52, 0xe2, 0xd3, 0x2b, 0x3e, 0xee, 0xd5,
]));

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
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
        load_internal_asset!(
            app,
            SPR_SHADER_HANDLE,
            "shaders/spr_shader.wgsl",
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
        SPR_SHADER_HANDLE.into()
    }

    fn deferred_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_SHADER_HANDLE.into()
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
    #[texture(0)]
    #[sampler(1)]
    pub color: Handle<Image>,
}

impl Material for SprTrueColorMaterial {
    fn alpha_mode(&self) -> bevy::prelude::AlphaMode {
        bevy::prelude::AlphaMode::Blend
    }

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_SHADER_HANDLE.into()
    }

    fn deferred_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SPR_SHADER_HANDLE.into()
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

        if let Some(frag_descriptor) = &mut descriptor.fragment {
            frag_descriptor
                .shader_defs
                .push("SPR_TRUE_COLOR_PIPELINE".into());
        }

        Ok(())
    }
}
