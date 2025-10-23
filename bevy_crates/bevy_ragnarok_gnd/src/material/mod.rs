use bevy_asset::{Asset, AssetApp, Handle, load_internal_asset, uuid_handle};
use bevy_image::Image;
use bevy_pbr::{Material, MaterialPlugin};
use bevy_reflect::Reflect;
use bevy_render::{alpha::AlphaMode, render_resource::AsBindGroup, storage::ShaderStorageBuffer};
use bevy_shader::{Shader, ShaderRef};

const GND_SHADER_HANDLE: Handle<Shader> = uuid_handle!("b7fa811a-e840-469e-b972-91bb81c55dfd");

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Asset
            .init_asset::<GndMaterial>()
            .register_asset_reflect::<GndMaterial>()
            // Material Plugin
            .add_plugins(MaterialPlugin::<GndMaterial>::default());

        // Shader handles
        load_internal_asset!(
            app,
            GND_SHADER_HANDLE,
            "shaders/gnd_shader.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Clone, Asset, Reflect, AsBindGroup)]
#[bindless(index_table(range(0..5)))]
pub struct GndMaterial {
    #[texture(0)]
    #[sampler(1)]
    #[dependency]
    pub texture: Handle<Image>,
    #[storage(2, binding_array(10), read_only)]
    pub cube_faces: Handle<ShaderStorageBuffer>,
    #[storage(3, binding_array(11), read_only)]
    pub surface_ids: Handle<ShaderStorageBuffer>,
    #[storage(4, binding_array(12), read_only)]
    pub surfaces: Handle<ShaderStorageBuffer>,
}

impl Material for GndMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn prepass_vertex_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn vertex_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy_pbr::MaterialPipeline,
        descriptor: &mut bevy_render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy_mesh::MeshVertexBufferLayoutRef,
        _key: bevy_pbr::MaterialPipelineKey<Self>,
    ) -> bevy_ecs::error::Result<(), bevy_render::render_resource::SpecializedMeshPipelineError>
    {
        descriptor.label = Some(
            format!(
                "gnd_{}",
                descriptor
                    .label
                    .as_ref()
                    .map(|label| label.as_ref())
                    .unwrap_or("shader")
            )
            .into(),
        );
        Ok(())
    }
}
