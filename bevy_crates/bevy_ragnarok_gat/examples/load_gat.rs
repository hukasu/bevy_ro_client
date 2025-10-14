//! Loads a Gat and builds it

use bevy::{
    DefaultPlugins,
    app::{App, Startup, Update},
    asset::{
        AssetApp, AssetServer,
        io::{AssetSourceBuilder, AssetSourceId},
    },
    camera::{Camera, Camera3d},
    ecs::{
        children,
        component::Component,
        query::With,
        spawn::SpawnRelated,
        system::{Commands, Res, ResMut, Single},
    },
    gizmos::{GizmoAsset, retained::Gizmo},
    input::{ButtonInput, keyboard::KeyCode},
    light::DirectionalLight,
    math::Vec3,
    scene::SceneSpawner,
    time::Time,
    transform::components::Transform,
    ui::{FlexDirection, Node, Val, widget::Text},
};
use bevy_ragnarok_gat::debug::{ToggleGatAabbs, ToggleGatQuads};

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

    app.add_plugins(DefaultPlugins);
    app.add_plugins(bevy_ragnarok_gat::plugin::Plugin);
    app.add_plugins(bevy_ragnarok_quad_tree::plugin::Plugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, debug);

    app.run();
}

#[derive(Component)]
struct World;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(Vec3::new(0., 250., 250.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::new(0., 5., 5.)).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let world = commands
        .spawn((
            World,
            // Convertion from Ragnarok's Y-down system to Bevy's Y-Up system
            Transform::from_scale(Vec3::new(1., -1., -1.)),
        ))
        .id();

    scene_spawner.spawn_as_child(
        asset_server.load_with_settings("data/prontera.gat#Scene", |settings: &mut f32| {
            // Ragnarok's usual world scale is 5. units
            *settings = 5.;
        }),
        world,
    );

    let mut world_axis = GizmoAsset::new();
    world_axis.axes(Transform::default(), 1.);
    commands.spawn((
        Gizmo {
            handle: asset_server.add(world_axis),
            ..Default::default()
        },
        Transform::default(),
    ));

    commands.spawn((
        Node {
            top: Val::Px(0.),
            left: Val::Px(0.),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        children![
            (Text::new("1: Toggle Aabb")),
            (Text::new("2: Toggle Quads"))
        ],
    ));
}

fn debug(
    mut commands: Commands,
    mut camera: Single<&mut Transform, With<Camera>>,
    key: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    const SCROLL_SPEED: f32 = 125.;
    if key.just_pressed(KeyCode::Digit1) {
        commands.trigger(ToggleGatAabbs);
    }
    if key.just_pressed(KeyCode::Digit2) {
        commands.trigger(ToggleGatQuads);
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
