mod components;
mod events;

use bevy::{
    app::Plugin,
    asset::{AssetServer, Handle},
    core::Name,
    ecs::query::QuerySingleError,
    prelude::{Commands, DespawnRecursiveExt, Entity, Query, Res, SpatialBundle, Trigger, With},
    scene::Scene,
};

pub use self::{
    components::World,
    events::{LoadWorld, UnloadWorld},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Observers
            .observe(load_world)
            .observe(unload_world);
    }
}

fn load_world(trigger: Trigger<LoadWorld>, mut commands: Commands, asset_loader: Res<AssetServer>) {
    // Spawn new world
    let next_world_handle: Handle<Scene> = asset_loader.load(trigger.event().world.to_string());
    commands.spawn((
        Name::new(trigger.event().world.clone()),
        World,
        next_world_handle,
        SpatialBundle::default(),
    ));
}

fn unload_world(
    _trigger: Trigger<UnloadWorld>,
    mut commands: Commands,
    worlds: Query<(Entity, &Name), With<World>>,
) {
    match worlds.get_single() {
        Ok((world, name)) => {
            bevy::log::trace!("Unloading world {:?}", name);
            commands.entity(world).despawn_recursive();
        }
        Err(QuerySingleError::NoEntities(err)) => {
            bevy::log::trace!(
                "Triggered `unload_world` but there were no Worlds loaded. {:?}",
                err
            );
        }
        Err(QuerySingleError::MultipleEntities(err)) => {
            bevy::log::error!(
                "Triggered `unload_world` but there were multiple worlds loaded. '{:?}'",
                err
            );
        }
    }
}
