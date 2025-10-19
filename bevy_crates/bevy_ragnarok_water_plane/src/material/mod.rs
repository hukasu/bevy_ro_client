pub(crate) mod plugin;

use bevy_asset::{Asset, Handle, uuid_handle};
use bevy_image::Image;
use bevy_mesh::MeshVertexBufferLayoutRef;
use bevy_pbr::{Material, MaterialPipeline, MaterialPipelineKey};
use bevy_reflect::Reflect;
use bevy_render::{
    alpha::AlphaMode,
    render_resource::{
        AsBindGroup, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    },
};
use bevy_shader::{Shader, ShaderRef};

const WATER_PLANE_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("13c76198-ee09-4c50-bbe4-5609d880269e");

#[derive(Debug, Clone, Asset, Reflect, AsBindGroup)]
#[bind_group_data(WaterPlaneMaterialKey)]
pub struct WaterPlaneMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub wave: Wave,
    pub opaque: bool,
}

impl Material for WaterPlaneMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        if self.opaque {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        }
    }

    fn vertex_shader() -> ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.label = Some(
            format!(
                "water_plane_{}",
                descriptor
                    .label
                    .as_ref()
                    .map(|label| label.as_ref())
                    .unwrap_or("shader")
            )
            .into(),
        );

        if let Some(frag_descriptor) = &mut descriptor.fragment
            && key.bind_group_data.opaque
        {
            frag_descriptor
                .shader_defs
                .push("OPAQUE_WATER_PLANE".into());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct Wave {
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_pitch: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WaterPlaneMaterialKey {
    opaque: bool,
}

impl From<&WaterPlaneMaterial> for WaterPlaneMaterialKey {
    fn from(value: &WaterPlaneMaterial) -> Self {
        Self {
            opaque: value.opaque,
        }
    }
}
