#[cfg(feature = "debug")]
pub mod debug;
mod loader;
mod resources;

use std::time::Duration;

use bevy_animation::{AnimationPlayer, graph::AnimationNodeIndex, prelude::AnimationTransitions};
use bevy_app::{Animation, PostUpdate, Update};
use bevy_asset::{AssetApp, Assets, Handle, weak_handle};
use bevy_ecs::{
    entity::Entity,
    observer::Trigger,
    query::{Changed, With},
    schedule::IntoScheduleConfigs,
    system::{Local, Query, Res, ResMut},
    world::OnAdd,
};
use bevy_math::{Vec2, Vec3, prelude::Plane3d};
use bevy_mesh::Mesh;
use bevy_render::view::{Visibility, VisibilitySystems};
use bevy_scene::SceneSpawner;
use resources::ActorSceneQueue;

use crate::{
    assets::ActorAnimations,
    components::{Actor, ActorAnchor, ActorFacing, ActorLayer, ActorPlayer},
};

use self::loader::AssetLoader;

const IDENTITY_PLANE_HANDLE: Handle<Mesh> = weak_handle!("e19c5b46-5ee0-452a-8d60-1d8b0da1fdf3");

pub struct Plugin {
    pub audio_path_prefix: std::path::PathBuf,
}

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Internal resources
            .init_resource::<ActorSceneQueue>()
            // Assets
            .init_asset::<ActorAnimations>()
            .register_asset_reflect::<ActorAnimations>()
            // AssetLoaders
            .register_asset_loader(AssetLoader {
                audio_path_prefix: self.audio_path_prefix.clone(),
            })
            // Types
            .register_type::<Actor>()
            .register_type::<ActorFacing>()
            .register_type::<ActorPlayer>()
            .register_type::<ActorLayer>()
            .register_type::<ActorAnchor>()
            // Observers
            .add_observer(spawn_actor_scene)
            .add_observer(start_idle_animation)
            // Systems
            .add_systems(
                Update,
                update_actor_scene_queue.run_if(update_actor_scene_queue_condition),
            )
            .add_systems(
                PostUpdate,
                update_visibility
                    .after(Animation)
                    .before(VisibilitySystems::CheckVisibility),
            );

        app.world_mut().resource_mut::<Assets<Mesh>>().insert(
            &IDENTITY_PLANE_HANDLE,
            Plane3d::new(Vec3::NEG_Z, Vec2::splat(0.5)).into(),
        );

        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);
    }
}

fn spawn_actor_scene(trigger: Trigger<OnAdd, Actor>, mut queue: ResMut<ActorSceneQueue>) {
    queue.push(trigger.target());
}

fn update_actor_scene_queue_condition(queue: Res<ActorSceneQueue>) -> bool {
    !queue.is_empty()
}

fn update_actor_scene_queue(
    mut queue: ResMut<ActorSceneQueue>,
    mut queue_double_buffer: Local<Vec<Entity>>,
    mut scene_spawner: ResMut<SceneSpawner>,
    actors: Query<&Actor>,
    actor_animations: Res<Assets<ActorAnimations>>,
) {
    for item in queue.drain(..) {
        if let Ok(actor) = actors.get(item) {
            if let Some(animation) = actor_animations.get(actor.actor.id()) {
                scene_spawner.spawn_as_child(animation.scene.clone(), item);
            } else {
                queue_double_buffer.push(item);
            }
        }
    }

    std::mem::swap(&mut queue.0, queue_double_buffer.as_mut());
}

fn start_idle_animation(
    trigger: Trigger<OnAdd, ActorPlayer>,
    mut actors: Query<(Entity, &mut AnimationPlayer, &mut AnimationTransitions), With<ActorPlayer>>,
) {
    let Ok((entity, mut player, mut transitions)) = actors.get_mut(trigger.target()) else {
        unreachable!("Should have components at this point.");
    };
    log::trace!("Starting animation of {}.", entity);
    transitions
        .play(player.as_mut(), AnimationNodeIndex::new(1), Duration::ZERO)
        .repeat();
}

fn update_visibility(mut actors: Query<(&mut Visibility, &ActorLayer), Changed<ActorLayer>>) {
    for (mut vis, layer) in actors.iter_mut() {
        match layer.active {
            true => *vis = Visibility::Inherited,
            false => *vis = Visibility::Hidden,
        }
    }
}
