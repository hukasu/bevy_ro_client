use std::marker::PhantomData;

use bevy::{
    asset::{load_internal_asset, Asset, AssetApp, Handle},
    pbr::{ExtendedMaterial, MaterialExtension, MaterialPlugin, StandardMaterial},
    prelude::Shader,
    reflect::Reflect,
    render::render_resource::AsBindGroup,
};

const RSM_MATERIAL_HANDLE: Handle<Shader> = Handle::weak_from_u128(u128::from_le_bytes([
    0x6f, 0x22, 0x2d, 0x4d, 0x29, 0x4c, 0x4d, 0x81, 0xb7, 0x66, 0x0f, 0x49, 0xd6, 0xd0, 0x33, 0x20,
]));

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_asset_reflect::<RsmMaterial>()
            .add_plugins(
                MaterialPlugin::<ExtendedMaterial<StandardMaterial, RsmMaterial>> {
                    prepass_enabled: true,
                    shadows_enabled: true,
                    _marker: PhantomData,
                },
            );
        load_internal_asset!(
            app,
            RSM_MATERIAL_HANDLE,
            "rsm_shader.wgsl",
            Shader::from_wgsl
        );
    }
}

#[derive(Debug, Clone, Asset, Reflect, AsBindGroup)]
pub struct RsmMaterial {}

impl MaterialExtension for RsmMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        RSM_MATERIAL_HANDLE.into()
    }

    fn deferred_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        RSM_MATERIAL_HANDLE.into()
    }
}
