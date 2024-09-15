mod components;
mod events;
mod loader;

use bevy::{
    asset::{AssetApp, AssetServer, Assets, Handle},
    core::Name,
    ecs::query::QuerySingleError,
    prelude::{Commands, DespawnRecursiveExt, Entity, Query, Res, ResMut, Trigger, With},
    scene::{Scene, SceneSpawner},
};

pub use ragnarok_rebuild_assets::rsw::Error;

use crate::tables::{name_table::NameTable, IndoorRsw};

use self::loader::AssetLoader;
pub use self::{
    components::{EnvironmentalLight, World},
    events::{LoadWorld, UnloadWorld},
    loader::RswSettings,
};

use super::paths;

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
            // Register AssetLoader
            .register_asset_loader(AssetLoader)
            // Observers
            .observe(load_world)
            .observe(unload_world);
    }
}

fn load_world(
    trigger: Trigger<LoadWorld>,
    mut scene_spawner: ResMut<SceneSpawner>,
    asset_loader: Res<AssetServer>,
    indoor_rsws: Res<IndoorRsw>,
    name_tables: Res<Assets<NameTable>>,
) {
    let world_to_load = &trigger.event().world;
    let is_indoor = {
        if let Some(name_table) = name_tables.get(&**indoor_rsws) {
            name_table.contains(&world_to_load.to_string())
        } else {
            bevy::log::error!("IndoorRsw name table does not exist.");
            false
        }
    };

    // Spawn new world
    let next_world_handle: Handle<Scene> = asset_loader.load_with_settings(
        format!("{}{}", paths::WORLD_FILES_FOLDER, world_to_load),
        move |settings: &mut RswSettings| {
            settings.is_indoor = is_indoor;
        },
    );
    scene_spawner.spawn(next_world_handle);
}

fn unload_world(
    _trigger: Trigger<UnloadWorld>,
    mut commands: Commands,
    worlds: Query<(Entity, &Name), With<components::World>>,
) {
    match worlds.get_single() {
        Ok((world, name)) => {
            bevy::log::trace!("Unloading world {:?}", name);
            commands.entity(world).despawn_recursive();
        }
        Err(QuerySingleError::NoEntities(_)) => {
            bevy::log::trace!("Triggered `unload_world` but there were no Worlds loaded.");
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            bevy::log::error!("Triggered `unload_world` but there were multiple worlds loaded.");
        }
    }
}
