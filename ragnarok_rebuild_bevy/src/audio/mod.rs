mod components;
mod events;
mod resources;

use bevy::{
    app::{Plugin, Update},
    asset::{AssetServer, Handle},
    audio::{
        AudioPlayer, AudioSink, AudioSinkPlayback, AudioSource, PlaybackMode, PlaybackSettings,
        SpatialScale, Volume,
    },
    prelude::{
        resource_changed, Commands, IntoScheduleConfigs, Name, Query, Res, Trigger, Visibility,
        With,
    },
};

pub use self::{
    components::{Bgm, Sound},
    events::{PlayBgm, PlaySound},
    resources::AudioSettings,
};

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
            .add_observer(play_bgm)
            .add_observer(play_sound);
    }
}

fn volume_changed(audio_settings: Res<AudioSettings>, mut bgms: Query<&mut AudioSink, With<Bgm>>) {
    bevy::log::trace!("Volume changed. {:?}", audio_settings);
    if let Ok(mut bgm) = bgms.single_mut() {
        bgm.set_volume(Volume::Linear(audio_settings.bgm_volume));
    };
}

fn play_bgm(
    trigger: Trigger<PlayBgm>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio_settings: Res<AudioSettings>,
) {
    let PlayBgm { track } = trigger.event();

    let bgm: Handle<AudioSource> = asset_server.load(format!("bgm://{}", track));

    commands.spawn((
        Name::new(track.to_string()),
        Bgm,
        AudioPlayer(bgm),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(audio_settings.bgm_volume),
            speed: 1.,
            paused: false,
            spatial: false,
            spatial_scale: None,
            muted: false,
        },
    ));
}

fn play_sound(
    trigger: Trigger<PlaySound>,
    mut commands: Commands,
    audio_settings: Res<AudioSettings>,
) {
    let PlaySound {
        name,
        track,
        position,
        volume,
        range,
    } = trigger.event();

    commands.spawn((
        Name::new(name.clone()),
        Sound,
        *position,
        Visibility::default(),
        AudioPlayer(track.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(volume * audio_settings.effects_volume),
            speed: 1.,
            paused: false,
            spatial: true,
            spatial_scale: Some(SpatialScale::new(5. / *range)),
            muted: false,
        },
    ));
}
