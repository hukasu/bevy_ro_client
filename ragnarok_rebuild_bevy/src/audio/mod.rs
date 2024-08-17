mod components;
mod events;
mod resources;

use bevy::{
    app::{Plugin, Update},
    asset::AssetServer,
    audio::{AudioBundle, AudioSink, AudioSinkPlayback, PlaybackMode, PlaybackSettings, Volume},
    core::Name,
    prelude::{resource_changed, Commands, IntoSystemConfigs, Query, Res, Trigger, With},
};

pub use self::{components::Bgm, events::PlayBgm, resources::AudioSettings};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        if !app.world_mut().contains_resource::<AudioSettings>() {
            app.init_resource::<AudioSettings>();
        }
        app
            // Resources
            .register_type::<AudioSettings>()
            // Systems
            .add_systems(
                Update,
                volume_changed.run_if(resource_changed::<AudioSettings>),
            )
            // Observers
            .observe(play_bgm);
    }
}

fn volume_changed(audio_settings: Res<AudioSettings>, bgms: Query<&AudioSink, With<Bgm>>) {
    bevy::log::trace!("Volume changed. {:?}", audio_settings);
    if let Ok(bgm) = bgms.get_single() {
        bgm.set_volume(audio_settings.bgm_volume);
    };
}

fn play_bgm(
    trigger: Trigger<PlayBgm>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
) {
    let PlayBgm { track } = trigger.event();

    let bgm = asset_server.load(format!("bgm://{}", track));

    commands.spawn((
        Name::new("Bgm"),
        Bgm,
        AudioBundle {
            source: bgm,
            settings: PlaybackSettings {
                mode: PlaybackMode::Loop,
                volume: Volume::new(audio_settings.bgm_volume),
                speed: 1.,
                paused: false,
                spatial: false,
                spatial_scale: None,
            },
        },
    ));
}
