pub(crate) mod plugin;

use bevy_asset::{Asset, AssetPath, Handle, embedded_path};
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
use bevy_shader::ShaderRef;

#[derive(Debug, Clone, Asset, Reflect, AsBindGroup)]
#[bind_group_data(WaterPlaneMaterialKey)]
pub struct WaterPlaneMaterial {
    #[texture(0, dimension = "2d_array")]
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
        AssetPath::from_path_buf(embedded_path!("shaders/water_plane_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/water_plane_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/water_plane_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/water_plane_shader.wgsl"))
            .with_source("embedded")
            .into()
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

        if let Some(frag_descriptor) = &mut descriptor.fragment {
            if frag_descriptor
                .shader_defs
                .contains(&bevy_shader::ShaderDefVal::Bool(
                    "PREPASS_PIPELINE".to_owned(),
                    true,
                ))
            {
                frag_descriptor.entry_point = Some("prepass_fragment".into());
            }
            if key.bind_group_data.opaque {
                frag_descriptor
                    .shader_defs
                    .push("OPAQUE_WATER_PLANE".into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Reflect, ShaderType)]
pub struct Wave {
    pub water_level: f32,
    pub wave_height: f32,
    pub wave_speed: f32,
    pub wave_pitch: f32,
    /// The framerate in a Ragnarok Online water plane is defined in cycles. To
    /// convert from cycles to frames per second do `60 / cycles`
    pub frames_per_second: f32,
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
