mod components;

use bevy::{
    animation::AnimationTarget,
    app::First,
    asset::{AssetServer, Assets},
    pbr::MeshMaterial3d,
    prelude::{Changed, ChildOf, Commands, Query, Res, With},
};
use ragnarok_act::components::{Actor, ActorLayer, ActorPlayer, SpritesheetIndex};
use ragnarok_spr::{
    assets::SpriteImages,
    components::Sprite,
    material::{SprIndexedMaterial, SprTrueColorMaterial, SprUniform},
};

pub use self::components::Entity;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<Entity>()
            // Systems
            .add_systems(First, update_sprite);
    }
}

fn update_sprite(
    mut commands: Commands,
    actor_layers: Query<
        (bevy::ecs::entity::Entity, &ActorLayer, &AnimationTarget),
        Changed<ActorLayer>,
    >,
    actor_players: Query<&ChildOf, With<ActorPlayer>>,
    actors: Query<&Sprite, With<Actor>>,
    sprites_images: Res<Assets<SpriteImages>>,
    asset_server: Res<AssetServer>,
) {
    for (actor_layer, layer, target) in actor_layers.iter() {
        let mut layer_commands = commands.entity(actor_layer);
        layer_commands.remove::<(
            MeshMaterial3d<SprIndexedMaterial>,
            MeshMaterial3d<SprTrueColorMaterial>,
        )>();

        let Ok(player) = actor_players.get(target.player) else {
            unreachable!("ActorPlayer must exist.");
        };

        if let Ok(sprite) = actors.get(player.parent()) {
            let Some(sprite_images) = sprites_images.get(sprite.0.id()) else {
                bevy::log::error!("SpriteImages {:?} does not exist.", sprite.0);
                return;
            };
            let uniform = SprUniform {
                uv_flip: layer.uv_flip as u32,
                tint: layer.tint,
            };
            match layer.spritesheet_index {
                SpritesheetIndex::Indexed(index) => {
                    layer_commands.insert(MeshMaterial3d(asset_server.add(SprIndexedMaterial {
                        uniform,
                        index_image: sprite_images.indexed_sprites[index].clone(),
                        palette: sprite_images.palette.clone(),
                    })));
                }
                SpritesheetIndex::TrueColor(index) => {
                    layer_commands.insert(MeshMaterial3d(asset_server.add(SprTrueColorMaterial {
                        uniform,
                        color: sprite_images.true_color_sprites[index].clone(),
                    })));
                }
            }
        } else {
            bevy::log::warn!("ActorLayer {} not grandchild of Actor.", actor_layer);
        }
    }
}
