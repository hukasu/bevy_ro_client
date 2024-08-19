mod components;

use bevy::{
    app::{Plugin, Startup},
    core::Name,
    prelude::{BuildChildren, Commands, Entity, OnAdd, Query, SpatialBundle, Trigger, With},
};

use ragnarok_rebuild_bevy::world::World;

use self::components::Game;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Startup system
            .add_systems(Startup, start_up)
            // Observers
            .observe(attach_world_to_game);
    }
}

fn start_up(mut commands: Commands) {
    commands
        .spawn((
            Name::new("RagnarokOnline"),
            components::Game,
            SpatialBundle::default(),
        ))
        .with_children(|builder| {
            builder.spawn((Name::new("Playing sounds"), SpatialBundle::default()));
            builder.spawn((Name::new("Actors"), SpatialBundle::default()));
        });
}

fn attach_world_to_game(
    trigger: Trigger<OnAdd, World>,
    mut commands: Commands,
    games: Query<Entity, With<Game>>,
) {
    let Ok(game) = games
        .get_single()
        .inspect_err(|err| bevy::log::error!("{err}"))
    else {
        return;
    };

    commands.entity(game).add_child(trigger.entity());
}
