use bevy_asset::{AssetApp, embedded_asset};
use bevy_pbr::MaterialPlugin;

use crate::material::{SprIndexedMaterial, SprMaterial, SprTrueColorMaterial};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.init_asset::<SprMaterial>()
            .register_asset_reflect::<SprMaterial>();

        app
            // Asset
            .init_asset::<SprIndexedMaterial>()
            .register_asset_reflect::<SprIndexedMaterial>()
            .init_asset::<SprTrueColorMaterial>()
            .register_asset_reflect::<SprTrueColorMaterial>()
            // Material Plugin
            .add_plugins(MaterialPlugin::<SprMaterial>::default())
            .add_plugins(MaterialPlugin::<SprIndexedMaterial>::default())
            .add_plugins(MaterialPlugin::<SprTrueColorMaterial>::default());

        // Shader handles
        embedded_asset!(app, "shaders/spr_shader.wgsl");
        embedded_asset!(app, "shaders/spr_vertex.wgsl");
        embedded_asset!(app, "shaders/spr_fragment.wgsl");
        embedded_asset!(app, "shaders/spr_prepass_fragment.wgsl");
    }
}
