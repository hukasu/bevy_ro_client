mod assets;
mod components;
mod events;
mod loader;

use core::f32;
use std::time::Duration;

use bevy::{
    app::Update,
    asset::{AssetApp, AssetServer, Assets, Handle, LoadState},
    core::Name,
    math::{Quat, Vec2, Vec3},
    prelude::{
        BuildChildren, Commands, DespawnRecursiveExt, Entity, EventReader, EventWriter,
        IntoSystemConfigs, Mesh, Plane3d, Query, Res, ResMut, SpatialBundle, Transform, Trigger,
        With, Without,
    },
    time::{Time, Timer},
};

use crate::{
    audio::PlaySound,
    materials::{SprIndexedMaterial, SprTrueColorMaterial},
};

use crate::assets::{paths, spr};

pub use self::{
    assets::Animation,
    components::{Actor, ActorFacing},
    events::LoadActor,
};
use self::{
    assets::{AnimationClip, AnimationEvent, AnimationFrame, AnimationLayer, AnimationLayerSprite},
    components::LoadingActor,
    events::ActorTimerTick,
    events::StartActor,
};

const IDENTITY_PLANE_HANDLE: Handle<Mesh> = Handle::weak_from_u128(0);

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Assets
            .init_asset::<Animation>()
            // AssetLoaders
            .register_asset_loader(loader::AssetLoader)
            // Events
            .add_event::<ActorTimerTick>()
            // Observers
            .observe(load_actor)
            .observe(start_animation)
            // Systems
            .add_systems(
                Update,
                check_actor_loading_state
                    .run_if(has_actors_loading)
                    .before(tick_animations),
            )
            .add_systems(Update, tick_animations.before(swap_animations))
            .add_systems(Update, swap_animations)
            // Types
            .register_type::<Actor>()
            .register_type::<ActorFacing>();

        app.world_mut().resource_mut::<Assets<Mesh>>().insert(
            &IDENTITY_PLANE_HANDLE,
            Plane3d::new(Vec3::Z, Vec2::splat(1.)).into(),
        );
    }
}

fn load_actor(trigger: Trigger<LoadActor>, mut commands: Commands, asset_server: Res<AssetServer>) {
    if trigger.entity() == Entity::PLACEHOLDER {
        bevy::log::error!("Could not spawn Actor because the event had no entity. Use `commands.trigger_targets`.");
    } else {
        // Spawning an empty entity for questions of ordering of
        // OnAdd observers
        let actor = commands.spawn_empty().id();
        // Pushing the empty actor to the parent, this will cause the
        // actor to have access to Parent on the OnAdd of the Actor components
        commands.entity(trigger.entity()).add_child(actor);
        // Adding actor components
        let actor_name = &trigger.event().actor;
        commands.entity(actor).insert((
            Name::new(format!("{}.act", actor_name)),
            SpatialBundle::default(),
            Actor {
                act: asset_server.load(format!("{}{}.act", paths::SPR_FILES_FOLDER, actor_name)),
                sprite: asset_server.load(format!("{}{}.spr", paths::SPR_FILES_FOLDER, actor_name)),
                palette: asset_server.load(format!(
                    "{}{}.spr#Palette",
                    paths::SPR_FILES_FOLDER,
                    actor_name
                )),
                facing: trigger.event().facing.unwrap_or_default(),
                clip: 0,
                frame: 0,
                timer: Timer::default(),
            },
            LoadingActor,
        ));
    }
}

fn has_actors_loading(actors: Query<Entity, With<LoadingActor>>) -> bool {
    !actors.is_empty()
}

fn check_actor_loading_state(
    mut commands: Commands,
    actors: Query<(Entity, &Actor), With<LoadingActor>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, actor) in actors.iter() {
        match asset_server.load_state(&actor.act) {
            LoadState::NotLoaded | LoadState::Loading => continue,
            LoadState::Loaded => commands.trigger_targets(StartActor, entity),
            LoadState::Failed(err) => {
                bevy::log::error!(
                    "Could not start Actor animation because asset failed to load with '{}'.",
                    err
                );
                commands.entity(entity).remove::<LoadingActor>();
                continue;
            }
        };
    }
}

fn start_animation(
    trigger: Trigger<StartActor>,
    mut commands: Commands,
    mut actors: Query<&mut Actor, With<LoadingActor>>,
    animations: Res<Assets<Animation>>,
) {
    let Ok(mut actor) = actors.get_mut(trigger.entity()) else {
        bevy::log::error!("Trying to start Actor animation on an entity that is not an Actor.");
        return;
    };

    bevy::log::trace!("Starting Actor animation.");
    commands.entity(trigger.entity()).remove::<LoadingActor>();

    let Some(animation) = animations.get(&actor.act) else {
        bevy::log::error!("Actor's Act was marked as Loaded but was not present on Assets.");
        return;
    };
    let clip = &animation.clips[actor.clip];

    actor.timer = Timer::new(
        Duration::from_secs_f32(clip.frame_time),
        bevy::time::TimerMode::Repeating,
    );
}

fn tick_animations(
    mut actors: Query<(Entity, &mut Actor), Without<LoadingActor>>,
    time: Res<Time>,
    mut event_writer: EventWriter<ActorTimerTick>,
    animations: Res<Assets<Animation>>,
) {
    let delta = time.delta();

    event_writer.send_batch(
        actors
            .iter_mut()
            .map(|(entity, mut actor)| {
                actor.timer.tick(delta);
                (entity, actor)
            })
            .filter(|(_, actor)| actor.timer.just_finished())
            .filter_map(|(entity, mut actor)| {
                let animation = animations.get(&actor.act)?;
                let clip = &animation.clips.get(actor.clip)?;

                let times_ticked = usize::try_from(actor.timer.times_finished_this_tick()).ok()?;
                actor.frame = (actor.frame + times_ticked) % clip.frames.len();

                Some(ActorTimerTick { entity })
            }),
    );
}

fn swap_animations(
    mut commands: Commands,
    mut event_reader: EventReader<ActorTimerTick>,
    actors: Query<(&Actor, &Transform), Without<LoadingActor>>,
    animations: Res<Assets<Animation>>,
    sprites: Res<Assets<spr::Sprite>>,
    mut sprites_indexed: ResMut<Assets<SprIndexedMaterial>>,
    mut sprites_true_color: ResMut<Assets<SprTrueColorMaterial>>,
) {
    for actor_id in event_reader.read() {
        let Ok((actor, actor_transform)) = actors.get(actor_id.entity) else {
            bevy::log::error!("An event to swap Actor's sprites had an inexistent Entity.");
            continue;
        };
        let Some(animation) = animations.get(&actor.act) else {
            continue;
        };
        let clip = &animation.clips[actor.clip];
        let frame = &clip.frames[actor.frame];

        commands.entity(actor_id.entity).despawn_descendants();

        commands.entity(actor_id.entity).with_children(|builder| {
            for (i, layer) in frame.layers.iter().enumerate() {
                let Some(actor_spritesheet) = sprites.get(&actor.sprite) else {
                    return;
                };

                let mut entity_commands = match layer.sprite {
                    AnimationLayerSprite::Indexed(id) => {
                        let Some(index_image) = actor_spritesheet.indexed_sprites.get(id).cloned()
                        else {
                            return;
                        };
                        let handle = sprites_indexed.add(SprIndexedMaterial {
                            index_image,
                            palette: actor.palette.clone(),
                        });
                        builder.spawn((IDENTITY_PLANE_HANDLE, handle))
                    }
                    AnimationLayerSprite::TrueColor(id) => {
                        let Some(color_texture) =
                            actor_spritesheet.true_color_sprites.get(id).cloned()
                        else {
                            return;
                        };
                        let handle = sprites_true_color.add(SprTrueColorMaterial {
                            color: color_texture,
                        });
                        builder.spawn((IDENTITY_PLANE_HANDLE, handle))
                    }
                };
                entity_commands.insert((
                    Name::new(format!("Layer{}", i)),
                    SpatialBundle {
                        transform: Transform::from_rotation(Quat::from_euler(
                            bevy::math::EulerRot::XYZ,
                            -f32::consts::FRAC_PI_6,
                            0.,
                            layer.rotation,
                        ))
                        .with_scale(layer.scale.extend(1.)),
                        ..Default::default()
                    },
                ));
            }
        });

        if let Some(AnimationEvent::Sound(sound)) = &frame.event {
            let sound_path = if let Some(sound_path) = sound.path() {
                sound_path.to_string()
            } else {
                "sound".to_owned()
            };
            commands.trigger(PlaySound {
                name: sound_path,
                track: sound.clone(),
                position: *actor_transform,
                volume: 1.,
                range: 50.,
            });
        }
    }
}
