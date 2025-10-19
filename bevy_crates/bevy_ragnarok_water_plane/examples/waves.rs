use bevy::{
    DefaultPlugins,
    app::{App, Startup},
    camera::Camera3d,
    ecs::system::Commands,
    light::DirectionalLight,
    math::Vec3,
    transform::components::Transform,
};
use bevy_asset::{
    AssetApp,
    io::{AssetSourceBuilder, AssetSourceId},
};
use bevy_ragnarok_water_plane::WaterPlane;

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

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(-5., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        WaterPlane {
            water_level: 0.,
            water_type: 0,
            wave_height: 0.25,
            wave_speed: 0.5,
            wave_pitch: 2.5,
            texture_cyclical_interval: 3,
        },
        Transform::from_scale(Vec3::new(5., -1., -5.)),
    ));
}
