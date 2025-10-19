use bevy::{DefaultPlugins, light::DirectionalLight};
use bevy_app::{App, Startup};
use bevy_asset::{Assets, Handle, RenderAssetUsages};
use bevy_camera::Camera3d;
use bevy_ecs::system::{Commands, ResMut};
use bevy_image::Image;
use bevy_math::{Vec2, Vec3, primitives::Plane3d};
use bevy_mesh::{Mesh, Mesh3d, MeshBuilder, Meshable};
use bevy_ragnarok_water_plane::{
    WaterPlane,
    material::{WaterPlaneMaterial, Wave},
};
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_transform::components::Transform;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins);
    app.add_plugins(bevy_ragnarok_water_plane::plugin::Plugin);

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<WaterPlaneMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(-5., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let plane = meshes.add(
        Plane3d::new(Vec3::NEG_Y, Vec2::splat(2.))
            .mesh()
            .subdivisions(12)
            .build(),
    );

    let white_image = images.add(Image::new_fill(
        Extent3d {
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    ));

    let water_plane_materials: [Handle<WaterPlaneMaterial>; 32] = std::array::from_fn(|_| {
        materials.add(WaterPlaneMaterial {
            texture: white_image.clone(),
            wave: Wave {
                wave_height: 0.25,
                wave_speed: 0.5,
                wave_pitch: 2.5,
            },
            opaque: false,
        })
    });

    commands.spawn((
        Mesh3d(plane),
        WaterPlane::new(water_plane_materials, 32),
        Transform::from_scale(Vec3::new(1., -1., -1.)),
    ));
}
