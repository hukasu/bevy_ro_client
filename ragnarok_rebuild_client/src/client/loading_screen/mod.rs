mod components;
mod resources;

use bevy::{
    app::Startup,
    asset::{AssetServer, Assets},
    color::Color,
    image::{Image, ImageLoaderSettings, ImageSampler},
    prelude::{
        BuildChildren, ChildBuild, Commands, DespawnRecursiveExt, Entity, ImageNode, OnEnter,
        OnExit, Res, Single, With,
    },
    ui::{AlignSelf, BackgroundColor, FlexDirection, Node, PositionType, Val, ZIndex},
};

use components::LoadingScreen;
use ragnarok_rebuild_bevy::assets::paths;

use crate::states::GameState;

use self::resources::LoadingScreens;

const MAXIMUM_LOADING_SCREENS: usize = 16;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Reflect
            .register_type::<LoadingScreen>()
            .register_type::<LoadingScreens>()
            // Systems
            .add_systems(Startup, collect_loading_screens)
            .add_systems(OnEnter(GameState::MapChange), show_loading_screen)
            .add_systems(OnExit(GameState::MapChange), hide_loading_screen);
    }
}

fn collect_loading_screens(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(LoadingScreens {
        loading_screens: (0..MAXIMUM_LOADING_SCREENS)
            .map(|id| {
                asset_server.load_with_settings(
                    format!("{}loading{:02}.jpg", paths::TEXTURE_FILES_FOLDER, id),
                    |settings: &mut ImageLoaderSettings| {
                        settings.sampler = ImageSampler::linear();
                    },
                )
            })
            .collect(),
    });
}

fn show_loading_screen(
    mut commands: Commands,
    loading_screens: Res<LoadingScreens>,
    images: Res<Assets<Image>>,
) {
    let valid_loading_screens = loading_screens
        .loading_screens
        .iter()
        .filter_map(|id| images.get(id.id()).map(|image| (id, image)))
        .collect::<Vec<_>>();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Absolute,
                ..Default::default()
            },
            ZIndex(i32::MAX),
            BackgroundColor(Color::BLACK),
            LoadingScreen,
        ))
        .with_children(|parent| {
            parent.spawn(Node {
                flex_direction: FlexDirection::Row,
                ..Default::default()
            });
        })
        .with_child((
            Node {
                height: Val::Percent(100.),
                aspect_ratio: Some(
                    valid_loading_screens[0].1.width() as f32
                        / valid_loading_screens[0].1.height() as f32,
                ),
                align_self: AlignSelf::Center,
                ..Default::default()
            },
            ImageNode {
                image: valid_loading_screens[0].0.clone(),
                ..Default::default()
            },
        ));
}

fn hide_loading_screen(
    mut commands: Commands,
    loading_screen: Single<Entity, With<LoadingScreen>>,
) {
    commands.entity(*loading_screen).despawn_recursive();
}
