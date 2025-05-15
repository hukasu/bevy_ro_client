use bevy::{
    audio::{AudioPlayer, PlaybackMode, PlaybackSettings, SpatialScale, Volume},
    prelude::{ChildOf, Commands, Query, Res, Transform, Trigger},
};
use ragnarok_act::events::ActorSound;
use ragnarok_rebuild_bevy::audio::{AudioSettings, Sound};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_observer(play_actor_sound);
    }
}

fn play_actor_sound(
    trigger: Trigger<ActorSound>,
    mut commands: Commands,
    transforms: Query<&Transform>,
    parents: Query<&ChildOf>,
    audio_settings: Res<AudioSettings>,
) {
    let event = trigger.event();

    // The animation player is 1 level deeper than the entity
    let parent = parents
        .get(trigger.target())
        .map(|childof| childof.parent())
        .unwrap_or(trigger.target());

    commands.spawn((
        event.name().clone(),
        Sound,
        transforms.get(parent).copied().unwrap_or_default(),
        AudioPlayer(event.sound().clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(audio_settings.effects_volume),
            speed: 1.,
            paused: false,
            spatial: true,
            spatial_scale: Some(SpatialScale::new(5. / 50.)),
            muted: false,
        },
    ));
}
