use bevy::{
    asset::{load_internal_asset, weak_handle, Asset, AssetApp, Handle},
    pbr::{Material, MaterialPlugin},
    prelude::{AlphaMode, Image, Shader},
    reflect::Reflect,
    render::render_resource::{AsBindGroup, ShaderType},
};

const WATER_PLANE_SHADER_HANDLE: Handle<Shader> =
    weak_handle!("13c76198-ee09-4c50-bbe4-5609d880269e");

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Asset
            .init_asset::<WaterPlaneMaterial>()
            .register_asset_reflect::<WaterPlaneMaterial>()
            // Material Plugin
            .add_plugins(MaterialPlugin::<WaterPlaneMaterial>::default());

        // Shader handles
        load_internal_asset!(
            app,
            WATER_PLANE_SHADER_HANDLE,
            "shaders/water_plane_shader.wgsl",
            Shader::from_wgsl
        );
    }
}

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

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn deferred_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        WATER_PLANE_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        if let Some(frag_descriptor) = &mut descriptor.fragment {
            if _key.bind_group_data.opaque {
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
