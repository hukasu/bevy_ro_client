use std::sync::Arc;

use bevy::{
    app::{PostStartup, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    audio::SpatialListener,
    core::Name,
    ecs::{
        entity::Entity,
        query::With,
        schedule::common_conditions::resource_exists,
        system::{Commands, Query, Res, ResMut},
    },
    math::Vec3,
    prelude::{Deref, DerefMut, IntoSystemConfigs, NextState, Resource, Visibility},
    text::Font,
    transform::components::Transform,
};
use bevy_flycam::FlyCam;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use ragnarok_rebuild_bevy::assets::{
    act::{ActorFacing, LoadActor},
    rsw::LoadWorld,
};

use crate::{client::entities, states::GameState};

const FONT_NAME: &str = "SCDream4";

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(WorldInspectorPlugin::default())
            .insert_resource(TeleportTextBox(String::new()))
            .add_systems(Startup, init_font_loading)
            .add_systems(
                Update,
                check_loading_font.run_if(resource_exists::<LoadingFont>),
            )
            .add_systems(Update, teleport_windows);

        app.add_plugins(bevy_flycam::prelude::PlayerPlugin)
            .insert_resource(bevy_flycam::prelude::MovementSettings {
                sensitivity: 0.00015, // default: 0.00012
                speed: 24.0,          // default: 12.0
            })
            .add_systems(PostStartup, add_listener_to_fly_cam);

        // app.add_plugins(iyes_perf_ui::PerfUiPlugin)
        //     .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        //     .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        //     .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin);

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
    bevy_inspector_egui::bevy_egui::egui::Window::new("Teleport").show(contexts.ctx_mut(), |ui| {
        ui.label("Destination");
        let text = (**text_box).clone();
        if ui.text_edit_singleline(&mut **text_box).lost_focus() && !text.is_empty() {
            commands.trigger(LoadWorld { world: text.into() });
            next_state.set(GameState::MapChange);
            text_box.clear();
        }
    });
}

fn add_listener_to_fly_cam(mut commands: Commands, flycams: Query<Entity, With<FlyCam>>) {
    let Ok(flycam) = flycams.get_single() else {
        bevy::log::error!("Zero or more than one FlyCam present.");
        return;
    };

    commands.entity(flycam).insert(SpatialListener::default());
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

    contexts.ctx_mut().set_fonts(font_definitons);
}

fn spawn_palette(mut commands: Commands) {
    let world_scale = 32. / 5.;
    let actor = commands
        .spawn((
            Name::new("anubis"),
            Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(1. / world_scale)),
            Visibility::default(),
            entities::Entity {},
        ))
        .id();
    commands.trigger_targets(
        LoadActor {
            actor: "몬스터/anubis".to_owned(),
            facing: Some(ActorFacing::SouthWest),
        },
        actor,
    );
    let actor = commands
        .spawn((
            Name::new("poring"),
            Transform::from_xyz(0., 0., 0.).with_scale(Vec3::splat(1. / world_scale)),
            Visibility::default(),
            entities::Entity {},
        ))
        .id();
    commands.trigger_targets(
        LoadActor {
            actor: "몬스터/poring".to_owned(),
            facing: Some(ActorFacing::SouthWest),
        },
        actor,
    );

    let actor = commands
        .spawn((
            Name::new("ghostring"),
            Transform::from_xyz(0., 0., -25.).with_scale(Vec3::splat(1. / world_scale)),
            Visibility::default(),
            entities::Entity {},
        ))
        .id();
    commands.trigger_targets(
        LoadActor {
            actor: "몬스터/ill_ghostring".to_owned(),
            facing: Some(ActorFacing::SouthWest),
        },
        actor,
    );
    let actor = commands
        .spawn((
            Name::new("ice_titan"),
            Transform::from_xyz(0., 0., -50.).with_scale(Vec3::splat(1. / world_scale)),
            Visibility::default(),
            entities::Entity {},
        ))
        .id();
    commands.trigger_targets(
        LoadActor {
            actor: "몬스터/ice_titan".to_owned(),
            facing: Some(ActorFacing::SouthWest),
        },
        actor,
    );
    let actor = commands
        .spawn((
            Name::new("4_f_01"),
            Transform::from_xyz(0., 0., -75.).with_scale(Vec3::splat(1. / world_scale)),
            Visibility::default(),
            entities::Entity {},
        ))
        .id();
    commands.trigger_targets(
        LoadActor {
            actor: "npc/4_f_01".to_owned(),
            facing: Some(ActorFacing::SouthWest),
        },
        actor,
    );
}
