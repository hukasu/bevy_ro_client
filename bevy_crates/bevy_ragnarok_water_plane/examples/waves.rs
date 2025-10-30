use bevy::{
    DefaultPlugins,
    app::{App, Startup},
    asset::{
        AssetApp, Assets,
        io::{AssetSourceBuilder, AssetSourceId},
    },
    camera::{Camera3d, visibility::Visibility},
    ecs::system::{Commands, ResMut},
    gizmos::{GizmoAsset, retained::Gizmo},
    light::DirectionalLight,
    math::Vec3,
    transform::components::Transform,
};
use bevy_ragnarok_water_plane::{WaterPlaneAsset, WaterPlaneBuilder};

fn main() {
    let mut app = App::new();

    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::platform_default("example_assets", None),
    );
    app.add_plugins(DefaultPlugins);
    app.add_plugins(bevy_ragnarok_water_plane::plugin::Plugin {
        texture_prefix: "".into(),
    });

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    mut water_planes: ResMut<Assets<WaterPlaneAsset>>,
    mut gizmos: ResMut<Assets<GizmoAsset>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 25., 25.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(-5., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mut gizmo = GizmoAsset::new();
    gizmo.axes(Transform::default(), 1.);
    commands.spawn((
        Gizmo {
            handle: gizmos.add(gizmo),
            ..Default::default()
        },
        Transform::default(),
    ));

    let water_plane = WaterPlaneAsset {
        water_level: 0.,
        water_type: 0,
        wave_height: 1.,
        wave_speed: 2.,
        wave_pitch: 50.,
        texture_cyclical_interval: 3,
    };
    commands.spawn((
        WaterPlaneBuilder {
            width: 18,
            height: 18,
            water_plane: water_planes.add(water_plane),
        },
        Transform::from_scale(Vec3::new(2., -0.2, -2.)),
        Visibility::default(),
    ));
}
