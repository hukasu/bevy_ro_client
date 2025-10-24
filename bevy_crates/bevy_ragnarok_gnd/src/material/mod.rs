use bevy_asset::{Asset, AssetApp, AssetPath, Handle, embedded_asset, embedded_path};
use bevy_image::Image;
use bevy_pbr::{Material, MaterialPlugin};
use bevy_reflect::Reflect;
use bevy_render::{alpha::AlphaMode, render_resource::AsBindGroup, storage::ShaderStorageBuffer};
use bevy_shader::ShaderRef;

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
        embedded_asset!(app, "shaders/gnd_shader.wgsl");
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
    #[storage(5, binding_array(13), read_only)]
    pub normals: Handle<ShaderStorageBuffer>,
}

impl Material for GndMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn prepass_vertex_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/gnd_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn vertex_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/gnd_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/gnd_shader.wgsl"))
            .with_source("embedded")
            .into()
    }

    fn fragment_shader() -> ShaderRef {
        AssetPath::from_path_buf(embedded_path!("shaders/gnd_shader.wgsl"))
            .with_source("embedded")
            .into()
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
