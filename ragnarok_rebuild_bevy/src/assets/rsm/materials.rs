use std::marker::PhantomData;

use bevy::{
    asset::{load_internal_asset, weak_handle, Asset, AssetApp, Handle},
    image::Image,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    reflect::Reflect,
    render::{
        alpha::AlphaMode,
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, Face, RenderPipelineDescriptor, Shader, ShaderRef,
            SpecializedMeshPipelineError,
        },
        RenderDebugFlags,
    },
};

const RSM_VERTEX_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("6f222d4d-294c-4d81-b766-0f49d6d03320");
const RSM_PREPASS_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("c017affd-8ac8-40cc-8948-03aeff34d07e");
const RSM_FRAGMENT_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("d13565d8-8786-4a32-a975-1d5f63c5f7ec");

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
                debug_flags: RenderDebugFlags::empty(),
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
#[bind_group_data(RsmMaterialKey)]
pub struct RsmMaterial {
    /// Texture of the Rsm
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    /// Double sided materials are visible from both sides
    pub double_sided: bool,
    /// There can be models that have N numbers of negative scale axis,
    /// if there is 1 or 3 negative scale axis, this should be `true`
    pub inverse_scale: bool,
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
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let label = descriptor.label.get_or_insert("shader".into());
        *label = format!("rsm_{}", label).into();

        descriptor.primitive.cull_mode = if key.bind_group_data.double_sided {
            None
        } else if key.bind_group_data.inverse_scale {
            Some(Face::Front)
        } else {
            Some(Face::Back)
        };

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RsmMaterialKey {
    double_sided: bool,
    inverse_scale: bool,
}

impl From<&RsmMaterial> for RsmMaterialKey {
    fn from(value: &RsmMaterial) -> Self {
        Self {
            double_sided: value.double_sided,
            inverse_scale: value.inverse_scale,
        }
    }
}
