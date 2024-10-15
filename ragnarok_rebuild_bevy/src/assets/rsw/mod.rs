mod components;
mod events;
mod loader;
mod quad_tree;
mod resources;

use bevy::{
    app::Update,
    asset::{AssetApp, AssetServer, Handle},
    core::Name,
    prelude::{
        resource_exists, Children, Commands, DespawnRecursiveExt, Entity, IntoSystemConfigs, OnAdd,
        Query, Res, ResMut, Trigger, With,
    },
    scene::{Scene, SceneInstance, SceneSpawner},
};

pub use ragnarok_rebuild_assets::rsw::Error;

use crate::assets::{paths, rsm};

pub use self::{
    components::{
        AnimatedProp, EnvironmentalEffect, EnvironmentalLight, EnvironmentalSound, World,
    },
    events::{LoadWorld, WorldLoaded},
    quad_tree::QuadTree,
};
use self::{events::UnloadWorld, loader::AssetLoader, resources::LoadingWorld};

const UNNAMED_WORLD: &str = "Unnamed World";

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Register Types
            .register_type::<components::World>()
            .register_type::<components::DiffuseLight>()
            .register_type::<components::AnimatedProp>()
            .register_type::<components::EnvironmentalLight>()
            .register_type::<components::EnvironmentalSound>()
            .register_type::<components::EnvironmentalEffect>()
            // Register AssetLoader
            .register_asset_loader(AssetLoader)
            // Systems
            .add_systems(Update, wait_models.run_if(resource_exists::<LoadingWorld>))
            // Observers
            .observe(load_world)
            .observe(unload_world)
            .observe(world_added)
            .observe(world_loaded);

        #[cfg(feature = "audio")]
        app.add_systems(Update, play_environmental_audio);
    }
}

fn load_world(
    trigger: Trigger<LoadWorld>,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_loader: Res<AssetServer>,
) {
    let world_to_load = &trigger.event().world;

    // Spawn new world
    let next_world_handle: Handle<Scene> =
        asset_loader.load(format!("{}{}", paths::WORLD_FILES_FOLDER, world_to_load));
    scene_spawner.spawn(next_world_handle);
}

fn unload_world(
    trigger: Trigger<UnloadWorld>,
    mut commands: Commands,
    world_names: Query<&Name, With<components::World>>,
) {
    let name = world_names
        .get(trigger.entity())
        .map_or(UNNAMED_WORLD, |name| name.as_str());
    bevy::log::trace!("Unloading world {}", name);
    commands.entity(trigger.entity()).despawn_recursive();
}

fn world_added(trigger: Trigger<OnAdd, World>, mut commands: Commands) {
    commands.insert_resource(LoadingWorld {
        world: trigger.entity(),
    });
}

fn world_loaded(
    trigger: Trigger<WorldLoaded>,
    mut commands: Commands,
    children: Query<&Children>,
    worlds: Query<Entity, With<components::World>>,
    animated_props: Query<(&Children, &components::AnimatedProp)>,
) {
    let other_worlds = worlds
        .iter()
        .filter(|world| world.ne(&trigger.entity()))
        .collect::<Vec<_>>();
    if !other_worlds.is_empty() {
        commands.trigger_targets(UnloadWorld, other_worlds);
    }

    let Ok(world_children) = children.get(trigger.entity()) else {
        bevy::log::error!("Just loaded world had no children.");
        return;
    };
    let Some(models_container) = world_children.iter().find_map(|child| {
        children
            .get(*child)
            .ok()
            .filter(|container| animated_props.contains(container[0]))
    }) else {
        bevy::log::error!("World does not have a container with AnimatedProps.");
        return;
    };

    for (animated_prop_children, animation_properties) in animated_props.iter_many(models_container)
    {
        if animated_prop_children.is_empty() {
            continue;
        }
        commands.trigger_targets(
            rsm::StartPropAnimation {
                speed: animation_properties.animation_speed,
                mode: animation_properties.animation_type,
            },
            animated_prop_children.to_vec(),
        )
    }
}

fn wait_models(
    mut commands: Commands,
    scene_spawner: ResMut<SceneSpawner>,
    loading_world: Res<LoadingWorld>,
    children: Query<&Children>,
    animated_props: Query<&SceneInstance, With<AnimatedProp>>,
) {
    let Some(world_children) = children.get(loading_world.world).ok() else {
        bevy::log::error!("World had no children or children was empty");
        return;
    };
    let Some(models_container) = world_children.iter().find_map(|child| {
        children
            .get(*child)
            .ok()
            .filter(|container| animated_props.contains(container[0]))
    }) else {
        bevy::log::error!("World does not have a container with AnimatedProps.");
        return;
    };
    if animated_props
        .iter_many(models_container)
        .all(|prop| scene_spawner.instance_is_ready(**prop))
    {
        commands.remove_resource::<LoadingWorld>();
        commands.trigger_targets(WorldLoaded, loading_world.world);
    }
}

#[cfg(feature = "audio")]
fn play_environmental_audio(
    mut commands: Commands,
    worlds: Query<(Entity, &components::World)>,
    children: Query<&Children>,
    mut environmental_sounds: Query<&mut EnvironmentalSound>,
    time: Res<bevy::time::Time>,
) {
    let Ok((world, world_info)) = worlds.get_single() else {
        return;
    };
    if !world_info.has_sounds {
        return;
    }
    let Some(world_children) = children.get(world).ok() else {
        bevy::log::error!("World had no children or children was empty");
        return;
    };
    let Some(sounds_container) = world_children.iter().find_map(|child| {
        children
            .get(*child)
            .ok()
            .filter(|container| environmental_sounds.contains(container[0]))
    }) else {
        bevy::log::error!("World does not have a container with Sounds.");
        return;
    };

    let tick = time.delta();

    for sound_ref in sounds_container {
        let Ok(mut sound) = environmental_sounds.get_mut(*sound_ref) else {
            continue;
        };

        sound.cycle.tick(tick);

        if sound.cycle.just_finished() {
            commands.trigger(crate::audio::PlaySound {
                name: sound.name.clone(),
                track: sound.source.clone(),
                position: sound.position,
                volume: sound.volume,
                range: sound.range,
            });
        }
    }
}
