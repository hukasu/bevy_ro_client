use std::marker::PhantomData;

use bevy_asset::{Asset, AssetApp, Handle, embedded_asset};
use bevy_image::Image;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy_reflect::Reflect;
use bevy_render::{
    RenderDebugFlags,
    alpha::AlphaMode,
    render_resource::{AsBindGroup, Face, RenderPipelineDescriptor, SpecializedMeshPipelineError},
};
use bevy_shader::ShaderRef;

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
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
        embedded_asset!(app, "shaders/rsm_fragment_shader.wgsl");
        embedded_asset!(app, "shaders/rsm_prepass_shader.wgsl");
        embedded_asset!(app, "shaders/rsm_vertex_shader.wgsl");
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
    /// The material might have transparency when using TGA textures
    pub transparency: bool,
}

impl Material for RsmMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        if self.transparency {
            AlphaMode::Blend
        } else {
            AlphaMode::Mask(0.5)
        }
    }

    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_rsm/materials/shaders/rsm_vertex_shader.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_rsm/materials/shaders/rsm_vertex_shader.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_rsm/materials/shaders/rsm_fragment_shader.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "embedded://bevy_ragnarok_rsm/materials/shaders/rsm_prepass_shader.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let label = descriptor.label.get_or_insert("shader".into());
        *label = format!("rsm_{}", label).into();

        descriptor.primitive.cull_mode = if key.bind_group_data.double_sided {
            None
        } else if key.bind_group_data.inverted_scale {
            Some(Face::Front)
        } else {
            Some(Face::Back)
        };

        if key.bind_group_data.double_sided
            && let Some(frag) = &mut descriptor.fragment
        {
            frag.shader_defs.push("RSM_MATERIAL_DOUBLE_SIDED".into());
        }

        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RsmMaterialKey {
    double_sided: bool,
    inverted_scale: bool,
}

impl From<&RsmMaterial> for RsmMaterialKey {
    fn from(value: &RsmMaterial) -> Self {
        Self {
            double_sided: value.double_sided,
            inverted_scale: value.inverse_scale,
        }
    }
}
