use std::sync::Arc;

use bevy::{
    app::{PostStartup, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    ecs::{
        schedule::common_conditions::resource_exists,
        system::{Commands, Res, ResMut},
    },
    prelude::{Deref, DerefMut, IntoScheduleConfigs, Name, NextState, Resource, Visibility},
    text::Font,
    transform::components::Transform,
};

use bevy_inspector_egui::{
    bevy_egui::{EguiPlugin, EguiPrimaryContextPass},
    quick::WorldInspectorPlugin,
};

use bevy_ragnarok_act::Actor;
use bevy_ragnarok_spr::Sprite;
use ragnarok_rebuild_bevy::assets::paths;

use crate::client::{entities, states::GameState, world::ChangeMap};

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
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        bevy_inspector_egui::bevy_egui::egui::Window::new("Teleport").show(ctx, |ui| {
            ui.label("Destination");
            let text = (**text_box).clone();
            if ui.text_edit_singleline(&mut **text_box).lost_focus() && !text.is_empty() {
                next_state.set(GameState::MapChange);
                commands.trigger(ChangeMap::new(text));
                text_box.clear();
            }
        });
    }
}

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
    let mut font_definitions = bevy_inspector_egui::egui::FontDefinitions::default();
    font_definitions
        .font_data
        .insert(FONT_NAME.to_owned(), font_data);

    let font_family = bevy_inspector_egui::egui::FontFamily::Proportional;
    let Some(font_family_store) = font_definitions.families.get_mut(&font_family) else {
        return;
    };
    font_family_store.insert(0, FONT_NAME.to_owned());

    if let Ok(ctx) = contexts.ctx_mut() {
        ctx.set_fonts(font_definitions)
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
