use bevy::{
    app::{PostStartup, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    audio::SpatialListener,
    prelude::{
        resource_exists, Commands, Deref, DerefMut, Entity, IntoSystemConfigs, Query, Res, ResMut,
        Resource, With,
    },
    text::Font,
};

use bevy_flycam::FlyCam;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use ragnarok_rebuild_bevy::assets::rsw::LoadWorld;

const FONT_NAME: &str = "SCDream4";

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(WorldInspectorPlugin::default())
            .add_plugins(bevy_flycam::prelude::PlayerPlugin)
            .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin)
            .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
            .add_plugins(iyes_perf_ui::PerfUiPlugin)
            .insert_resource(bevy_flycam::prelude::MovementSettings {
                sensitivity: 0.00015, // default: 0.00012
                speed: 24.0,          // default: 12.0
            })
            .insert_resource(TeleportTextBox(String::new()))
            .add_systems(PostStartup, add_listener_to_fly_cam)
            .add_systems(Startup, (spawn_perf_ui, init_font_loading))
            .add_systems(
                Update,
                check_loading_font.run_if(resource_exists::<LoadingFont>),
            )
            .add_systems(Update, teleport_windows);
    }
}

#[derive(Debug, Resource, Deref, DerefMut)]
struct TeleportTextBox(pub String);

fn teleport_windows(
    mut contexts: bevy_inspector_egui::bevy_egui::EguiContexts,
    mut commands: Commands,
    mut text_box: ResMut<TeleportTextBox>,
) {
    bevy_inspector_egui::bevy_egui::egui::Window::new("Teleport").show(contexts.ctx_mut(), |ui| {
        ui.label("Destination");
        let text = (**text_box).clone();
        if ui.text_edit_singleline(&mut **text_box).lost_focus() && !text.is_empty() {
            commands.trigger(LoadWorld { world: text.into() });
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

fn spawn_perf_ui(mut commands: Commands) {
    commands.spawn(iyes_perf_ui::entries::PerfUiBundle::default());
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

    let font_box: &dyn ab_glyph::Font = &font.font;
    let font_data = bevy_inspector_egui::egui::FontData::from_owned(font_box.font_data().to_vec());
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
