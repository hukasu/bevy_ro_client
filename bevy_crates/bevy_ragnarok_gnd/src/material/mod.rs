use bevy_asset::{Asset, AssetApp, Handle, load_internal_asset, uuid_handle};
use bevy_image::Image;
use bevy_mesh::{Mesh, MeshVertexAttribute};
use bevy_pbr::{Material, MaterialPlugin};
use bevy_reflect::Reflect;
use bevy_render::{
    alpha::AlphaMode,
    render_resource::{AsBindGroup, VertexFormat},
    storage::ShaderStorageBuffer,
};
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
pub struct GndMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub color_texture: Handle<Image>,
    #[storage(2, read_only)]
    pub texture_uvs: Handle<ShaderStorageBuffer>,
}

impl GndMaterial {
    pub const TEXTURE_ID_VERTEX_ATTRIBUTE: MeshVertexAttribute =
        MeshVertexAttribute::new("TextureId", 1010101001, VertexFormat::Uint32);
}

impl Material for GndMaterial {
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }

    fn vertex_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn deferred_vertex_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        GND_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy_pbr::MaterialPipeline,
        descriptor: &mut bevy_render::render_resource::RenderPipelineDescriptor,
        layout: &bevy_mesh::MeshVertexBufferLayoutRef,
        _key: bevy_pbr::MaterialPipelineKey<Self>,
    ) -> bevy_ecs::error::Result<(), bevy_render::render_resource::SpecializedMeshPipelineError>
    {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Self::TEXTURE_ID_VERTEX_ATTRIBUTE.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
