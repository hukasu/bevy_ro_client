use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::{
        AssetApp, AssetServer, Assets, RenderAssetUsages,
        io::{AssetSourceBuilder, AssetSourceId},
    },
    camera::Camera3d,
    ecs::{
        entity::Entity,
        query::Without,
        system::{Commands, Populated, Res, ResMut},
    },
    light::DirectionalLight,
    math::{Vec2, Vec3, primitives::Plane3d},
    mesh::{Mesh3d, MeshBuilder, MeshTag, Meshable},
    pbr::MeshMaterial3d,
    render::storage::ShaderStorageBuffer,
    transform::components::Transform,
};
use bevy_ragnarok_spr::{Sprite, SpriteImages, material::SprMaterial};

fn main() {
    let mut app = App::new();

    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::platform_default("example_assets", None),
    );
    app.add_plugins(DefaultPlugins);

    app.add_plugins(bevy_ragnarok_spr::plugin::Plugin);
    app.add_plugins(bevy_ragnarok_pal::plugin::Plugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, make_material);

    app.run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 0., 15.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(0., 0., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mesh = asset_server.add(Plane3d::new(Vec3::NEG_Z, Vec2::splat(0.5)).mesh().build());

    commands.spawn((
        Mesh3d(mesh),
        Sprite(asset_server.load("poring.spr")),
        Transform::from_scale(Vec3::new(0.2, -0.2, -0.2)),
    ));
}

fn make_material(
    mut commands: Commands,
    sprites: Populated<(Entity, &Sprite), Without<MeshMaterial3d<SprMaterial>>>,
    sprite_images: Res<Assets<SpriteImages>>,
    mut sprite_materials: ResMut<Assets<SprMaterial>>,
    mut shader_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let mut uv_indexes = Vec::with_capacity(SprMaterial::UV_INDEX_STRIDE);
    uv_indexes.extend_from_slice(&0u32.to_le_bytes());
    let mut tint = Vec::with_capacity(SprMaterial::TINT_STRIDE);
    tint.extend_from_slice(&1f32.to_le_bytes());
    tint.extend_from_slice(&1f32.to_le_bytes());
    tint.extend_from_slice(&1f32.to_le_bytes());
    tint.extend_from_slice(&1f32.to_le_bytes());
    let mut flipped_uvs = Vec::with_capacity(SprMaterial::FLIPPED_UVS_STRIDE);
    flipped_uvs.extend_from_slice(&0u32.to_le_bytes());

    for (entity, sprite) in sprites {
        if let Some(images) = sprite_images.get(sprite.0.id()) {
            let mat = sprite_materials.add(SprMaterial {
                index_image: images.indexed_sprites.clone(),
                palette: images.palette.clone(),
                true_color_image: images.true_color_sprites.clone(),
                indexed_uvs: images.indexed_sprites_uvs.clone(),
                true_color_uvs: images.true_color_sprites_uvs.clone(),
                uv_index: shader_buffers.add(ShaderStorageBuffer::new(
                    &uv_indexes,
                    RenderAssetUsages::all(),
                )),
                tint: shader_buffers.add(ShaderStorageBuffer::new(&tint, RenderAssetUsages::all())),
                flipped_uvs: shader_buffers.add(ShaderStorageBuffer::new(
                    &flipped_uvs,
                    RenderAssetUsages::all(),
                )),
            });
            commands
                .entity(entity)
                .insert((MeshMaterial3d(mat), MeshTag(0)));
        }
    }
}
