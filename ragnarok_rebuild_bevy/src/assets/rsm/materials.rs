use std::marker::PhantomData;

use bevy::{
    asset::{load_internal_asset, Asset, AssetApp, Handle},
    image::Image,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    reflect::Reflect,
    render::{
        alpha::AlphaMode,
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, Shader, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

const RSM_VERTEX_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x6f222d4d294c4d81b7660f49d6d03320);
const RSM_PREPASS_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xc017affd8ac840cc894803aeff34d07e);
const RSM_FRAGMENT_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0xd13565d887864a32a9751d5f63c5f7ec);

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Asset
            .init_asset::<RsmMaterial>()
            .register_asset_reflect::<RsmMaterial>()
            // Material Plugin
            .add_plugins(MaterialPlugin::<RsmMaterial> {
                prepass_enabled: true,
                shadows_enabled: true,
                _marker: PhantomData,
            });
        load_internal_asset!(
            app,
            RSM_FRAGMENT_SHADER_HANDLE,
            "shaders/rsm_fragment_shader.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            RSM_PREPASS_SHADER_HANDLE,
            "shaders/rsm_prepass_shader.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            RSM_VERTEX_SHADER_HANDLE,
            "shaders/rsm_vertex_shader.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Debug, Clone, Asset, Reflect, AsBindGroup)]
pub struct RsmMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl Material for RsmMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn vertex_shader() -> ShaderRef {
        RSM_VERTEX_SHADER_HANDLE.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        RSM_VERTEX_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        RSM_FRAGMENT_SHADER_HANDLE.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        RSM_PREPASS_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let label = descriptor.label.get_or_insert("shader".into());
        *label = format!("rsm_{}", label).into();

        Ok(())
    }
}
