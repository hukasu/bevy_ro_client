use bevy::{
    app::Startup,
    asset::{AssetServer, Assets, Handle},
    camera::{Camera, Camera2d},
    color::Color,
    ecs::{children, component::Component, lifecycle::Insert, observer::On, spawn::SpawnRelated},
    image::{Image, ImageLoaderSettings, ImageSampler},
    prelude::{Commands, Entity, OnEnter, OnExit, Res, Single, With},
    ui::{
        widget::ImageNode, AlignSelf, BackgroundColor, FlexDirection, Node, PositionType,
        UiTargetCamera, Val, ZIndex,
    },
};

use super::states::GameState;

const MAXIMUM_LOADING_SCREENS: usize = 16;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // Systems
        app.add_systems(Startup, spawn_loading_screen)
            .add_systems(OnEnter(GameState::MapChange), show_loading_screen)
            .add_systems(OnExit(GameState::MapChange), hide_loading_screen);
        app.add_observer(replace_loading_screen);
    }
}

#[derive(Component)]
struct LoadingScreens {
    loading_screens: Vec<Handle<Image>>,
}

#[derive(Component)]
struct LoadingScreen;

#[derive(Component)]
struct LoadingScreenImage(usize);

#[derive(Component)]
struct LoadingScreenCamera;

/// Spawns a camera where the loading screens will be displayed. Also builds the Ui
/// hierarchy for displaying the loading screen.
///
/// This camera is initially disabled, and gets enabled on [`GameState::MapChange`].
fn spawn_loading_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = commands
        .spawn((
            Camera2d,
            Camera {
                order: 16,
                is_active: false,
                ..Default::default()
            },
            LoadingScreenCamera,
        ))
        .id();

    let loading_screens = LoadingScreens {
        loading_screens: (0..MAXIMUM_LOADING_SCREENS)
            .map(|id| {
                asset_server.load_with_settings(
                    format!("data/texture/유저인터페이스/loading{:02}.jpg", id),
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::linear();
                    },
                )
            })
            .collect(),
    };

    commands.spawn((
        LoadingScreen,
        loading_screens,
        UiTargetCamera(camera),
        Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            ..Default::default()
        },
        ZIndex(i32::MAX),
        BackgroundColor(Color::BLACK),
        children![(
            Node {
                height: Val::Percent(100.),
                align_self: AlignSelf::Center,
                flex_direction: FlexDirection::Row,
                flex_grow: 1.,
                ..Default::default()
            },
            children![(LoadingScreenImage(usize::MAX),)],
        )],
    ));
}

fn show_loading_screen(
    mut commands: Commands,
    loading_screen: Single<&LoadingScreens, With<LoadingScreen>>,
    loading_screen_image: Single<(Entity, &LoadingScreenImage)>,
    mut loading_screen_camera: Single<&mut Camera, With<LoadingScreenCamera>>,
) {
    loading_screen_camera.is_active = true;

    let (entity, loading_screen_image) = loading_screen_image.into_inner();

    let index = loading_screen_image.0.wrapping_add(1) % loading_screen.loading_screens.len();
    commands.entity(entity).insert(LoadingScreenImage(index));
}

fn hide_loading_screen(mut loading_screen_camera: Single<&mut Camera, With<LoadingScreenCamera>>) {
    loading_screen_camera.is_active = false;
}

fn replace_loading_screen(
    event: On<Insert, LoadingScreenImage>,
    mut commands: Commands,
    loading_screen: Single<&LoadingScreens, With<LoadingScreen>>,
    loading_screen_image: Single<&LoadingScreenImage>,
    images: Res<Assets<Image>>,
) {
    let index = loading_screen_image.0;
    let Some(image_handle) = loading_screen.loading_screens.get(index).cloned() else {
        return;
    };
    let Some(image) = images.get(image_handle.id()) else {
        return;
    };

    commands.entity(event.entity).insert((
        Node {
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            aspect_ratio: Some(image.aspect_ratio().ratio()),
            align_self: AlignSelf::Center,
            ..Default::default()
        },
        ImageNode {
            image: image_handle,
            ..Default::default()
        },
    ));
}
