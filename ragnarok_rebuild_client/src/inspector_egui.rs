use std::{path::PathBuf, sync::Arc};

use bevy::{
    app::{PostStartup, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    ecs::{
        schedule::common_conditions::resource_exists,
        system::{Commands, Res, ResMut},
    },
    input::common_conditions::input_just_pressed,
    prelude::{
        Camera, Deref, DerefMut, Entity, IntoScheduleConfigs, KeyCode, Name, NextState, Query,
        Resource, Single, Visibility, With,
    },
    text::Font,
    transform::components::Transform,
};

use bevy_flycam::FlyCam;
use bevy_inspector_egui::{
    bevy_egui::{EguiPlugin, EguiPrimaryContextPass},
    quick::WorldInspectorPlugin,
};

use ragnarok_act::components::Actor;
use ragnarok_rebuild_bevy::assets::paths;
use ragnarok_spr::components::Sprite;

use crate::client::{entities, resources::LoadingWorld, states::GameState};

const FONT_NAME: &str = "SCDream4";

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((EguiPlugin::default(), WorldInspectorPlugin::default()))
            .insert_resource(TeleportTextBox(String::new()))
            .add_systems(Startup, init_font_loading)
            .add_systems(
                Update,
                check_loading_font.run_if(resource_exists::<LoadingFont>),
            )
            .add_systems(EguiPrimaryContextPass, teleport_windows);

        // FlyCam
        app.add_plugins(bevy_flycam::NoCameraPlayerPlugin)
            .insert_resource(bevy_flycam::MovementSettings::default())
            .add_systems(
                Update,
                toggle_flycam.run_if(input_just_pressed(KeyCode::KeyF)),
            );

        app.add_plugins(iyes_perf_ui::PerfUiPlugin)
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin);

        app.add_systems(PostStartup, spawn_palette);
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct TeleportTextBox(pub String);

fn teleport_windows(
    mut contexts: bevy_inspector_egui::bevy_egui::EguiContexts,
    mut commands: Commands,
    mut text_box: ResMut<TeleportTextBox>,
    mut next_state: ResMut<NextState<GameState>>,
    asset_server: Res<AssetServer>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        bevy_inspector_egui::bevy_egui::egui::Window::new("Teleport").show(ctx, |ui| {
            ui.label("Destination");
            let text = (**text_box).clone();
            if ui.text_edit_singleline(&mut **text_box).lost_focus() && !text.is_empty() {
                next_state.set(GameState::MapChange);
                commands.insert_resource(LoadingWorld {
                    world: asset_server.load(PathBuf::from(paths::WORLD_FILES_FOLDER).join(text)),
                });
                text_box.clear();
            }
        });
    }
}

fn toggle_flycam(
    mut commands: Commands,
    camera: Single<Entity, With<Camera>>,
    flycams: Query<&FlyCam>,
) {
    if flycams.contains(*camera) {
        commands.entity(*camera).remove::<FlyCam>();
    } else {
        commands.entity(*camera).insert(FlyCam);
    }
}

// fn spawn_perf_ui(mut commands: Commands) {
//     commands.spawn(iyes_perf_ui::entries::PerfUiBundle::default());
// }

#[derive(Debug, Resource)]
struct LoadingFont(Handle<Font>);

fn init_font_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load(format!("system://font/{}.otf", FONT_NAME));
    commands.insert_resource(LoadingFont(font));
}

fn check_loading_font(
    mut commands: Commands,
    mut contexts: bevy_inspector_egui::bevy_egui::EguiContexts,
    loading_font: Res<LoadingFont>,
    fonts: Res<Assets<Font>>,
) {
    let Some(font) = fonts.get(&loading_font.0) else {
        return;
    };

    commands.remove_resource::<LoadingFont>();

    let font_data = Arc::new(bevy_inspector_egui::egui::FontData::from_owned(
        Vec::from_iter(font.data.iter().copied()),
    ));
    let mut font_definitons = bevy_inspector_egui::egui::FontDefinitions::default();
    font_definitons
        .font_data
        .insert(FONT_NAME.to_owned(), font_data);

    let font_family = bevy_inspector_egui::egui::FontFamily::Proportional;
    let Some(font_family_store) = font_definitons.families.get_mut(&font_family) else {
        return;
    };
    font_family_store.insert(0, FONT_NAME.to_owned());

    if let Ok(ctx) = contexts.ctx_mut() {
        ctx.set_fonts(font_definitons)
    };
}

fn spawn_palette(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("anubis"),
        Transform::from_xyz(0., 0., 0.),
        Visibility::default(),
        entities::Entity {},
        Actor {
            actor: asset_server.load(format!(
                "{}{}",
                paths::SPR_FILES_FOLDER,
                "몬스터/anubis.act"
            )),
        },
        Sprite(asset_server.load(format!(
            "{}{}",
            paths::SPR_FILES_FOLDER,
            "몬스터/anubis.spr"
        ))),
    ));
    commands.spawn((
        Name::new("poring"),
        Transform::from_xyz(0., 0., 0.),
        Visibility::default(),
        entities::Entity {},
        Actor {
            actor: asset_server.load(format!(
                "{}{}",
                paths::SPR_FILES_FOLDER,
                "몬스터/poring.act"
            )),
        },
        Sprite(asset_server.load(format!(
            "{}{}",
            paths::SPR_FILES_FOLDER,
            "몬스터/poring.spr"
        ))),
    ));

    commands.spawn((
        Name::new("ghostring"),
        Transform::from_xyz(0., 0., -25.),
        Visibility::default(),
        entities::Entity {},
        Actor {
            actor: asset_server.load(format!(
                "{}{}",
                paths::SPR_FILES_FOLDER,
                "몬스터/ill_ghostring.act"
            )),
        },
        Sprite(asset_server.load(format!(
            "{}{}",
            paths::SPR_FILES_FOLDER,
            "몬스터/ill_ghostring.spr"
        ))),
    ));
    commands.spawn((
        Name::new("ice_titan"),
        Transform::from_xyz(0., 0., -50.),
        Visibility::default(),
        entities::Entity {},
        Actor {
            actor: asset_server.load(format!(
                "{}{}",
                paths::SPR_FILES_FOLDER,
                "몬스터/ice_titan.act"
            )),
        },
        Sprite(asset_server.load(format!(
            "{}{}",
            paths::SPR_FILES_FOLDER,
            "몬스터/ice_titan.spr"
        ))),
    ));
    commands.spawn((
        Name::new("4_f_01"),
        Transform::from_xyz(0., 0., -75.),
        Visibility::default(),
        entities::Entity {},
        Actor {
            actor: asset_server.load(format!("{}{}", paths::SPR_FILES_FOLDER, "npc/4_f_01.act")),
        },
        Sprite(asset_server.load(format!("{}{}", paths::SPR_FILES_FOLDER, "npc/4_f_01.spr"))),
    ));
}
