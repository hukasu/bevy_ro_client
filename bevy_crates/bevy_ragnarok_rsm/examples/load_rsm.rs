//! Loads a Gat and builds it

use std::f32::consts::PI;

use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::{
        AssetApp, AssetServer,
        io::{AssetSourceBuilder, AssetSourceId},
    },
    camera::{Camera, Camera3d},
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Res, Single},
    },
    gizmos::{GizmoAsset, retained::Gizmo},
    input::{ButtonInput, keyboard::KeyCode},
    light::DirectionalLight,
    math::{Quat, Vec3},
    time::Time,
    transform::components::Transform,
};
use bevy_app::PluginGroup;
use bevy_camera::visibility::Visibility;
use bevy_ecs::hierarchy::ChildOf;
use bevy_image::{ImageFilterMode, ImagePlugin, ImageSamplerDescriptor};
use bevy_ragnarok_rsm::debug::{ToggleRsmEdges, ToggleRsmNormals};
use bevy_scene::SceneRoot;

fn main() {
    let mut app = App::new();

    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::default().with_reader(|| {
            #[expect(clippy::unwrap_used, reason = "This is on my TODO")]
            let grf =
                bevy_ragnarok_grf::AssetReader::new(std::path::Path::new("data.grf")).unwrap();
            Box::new(grf)
        }),
    );

    app.add_plugins(DefaultPlugins.set(ImagePlugin {
        default_sampler: ImageSamplerDescriptor {
            label: Some("default_linear_sampler".into()),
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            ..Default::default()
        },
    }));
    app.add_plugins(bevy_ragnarok_rsm::plugin::Plugin {
        texture_path_prefix: "data/texture/".into(),
    });

    app.add_systems(Startup, setup);
    app.add_systems(Update, debug);

    app.run();
}

#[derive(Component)]
struct World;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 25., 25.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(0., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let world = commands
        .spawn((
            World,
            Transform::from_rotation(Quat::from_rotation_x(PI)).with_scale(Vec3::splat(0.2)),
            Visibility::Inherited,
        ))
        .id();

    for (i, path) in [
        "data/model/prontera/prn_statue_08.rsm",
        "data/model/prt/trees_s_01.rsm2",
    ]
    .into_iter()
    .enumerate()
    {
        let scale_invert = if path.ends_with(".rsm2") {
            Vec3::new(1., -1., 1.)
        } else {
            Vec3::new(1., 1., 1.)
        };

        let model = asset_server.load(format!("{path}#Scene"));
        commands.spawn((
            Transform::from_translation(Vec3::new(50. * i as f32, 0., 0.)).with_scale(scale_invert),
            SceneRoot(model.clone()),
            ChildOf(world),
        ));
        commands.spawn((
            Transform::from_translation(Vec3::new(50. * i as f32, 0., 50.))
                .with_scale(Vec3::new(-1., 1., 1.) * scale_invert),
            SceneRoot(model.clone()),
            ChildOf(world),
        ));
    }

    let mut world_axis = GizmoAsset::new();
    world_axis.axes(Transform::default(), 1.);
    commands.spawn((
        Gizmo {
            handle: asset_server.add(world_axis),
            ..Default::default()
        },
        Transform::default(),
    ));
}

fn debug(
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera>>,
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    const SCROLL_SPEED: f32 = 25.;
    if key.just_pressed(KeyCode::Digit1) {
        commands.trigger(ToggleRsmEdges);
    }
    if key.just_pressed(KeyCode::Digit2) {
        commands.trigger(ToggleRsmNormals);
    }

    let delta = time.delta_secs();
    if key.pressed(KeyCode::KeyW) {
        camera.translation.z -= SCROLL_SPEED * delta;
    }
    if key.pressed(KeyCode::KeyA) {
        camera.translation.x -= SCROLL_SPEED * delta;
    }
    if key.pressed(KeyCode::KeyS) {
        camera.translation.z += SCROLL_SPEED * delta;
    }
    if key.pressed(KeyCode::KeyD) {
        camera.translation.x += SCROLL_SPEED * delta;
    }
}
