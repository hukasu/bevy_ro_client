use bevy::{
    audio::{AudioPlayer, PlaybackMode, PlaybackSettings, SpatialScale, Volume},
    ecs::observer::On,
    prelude::{ChildOf, Commands, Query, Res, Transform},
};
use bevy_ragnarok_act::events::ActorSound;
use ragnarok_rebuild_bevy::audio::{AudioSettings, Sound};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_observer(play_actor_sound);
    }
}

fn play_actor_sound(
    actor_sound: On<ActorSound>,
    mut commands: Commands,
    transforms: Query<&Transform>,
    parents: Query<&ChildOf>,
    audio_settings: Res<AudioSettings>,
) {
    // The animation player is 1 level deeper than the entity
    let parent = parents
        .get(actor_sound.trigger().animation_player)
        .map(|childof| childof.parent())
        .unwrap_or(actor_sound.trigger().animation_player);

    commands.spawn((
        actor_sound.name().clone(),
        Sound,
        transforms.get(parent).copied().unwrap_or_default(),
        AudioPlayer(actor_sound.sound().clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::Linear(audio_settings.effects_volume),
            speed: 1.,
            paused: false,
            spatial: true,
            spatial_scale: Some(SpatialScale::new(5. / 50.)),
            muted: false,
            ..Default::default()
        },
    ));
}
