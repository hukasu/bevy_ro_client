use bevy::{
    animation::AnimationPlayer,
    asset::Handle,
    ecs::{
        entity::Entity,
        query::{Added, With},
        system::Query,
    },
    scene::Scene,
};

use crate::assets::rsm;

pub fn start_animations(
    worlds_query: Query<Entity, (With<super::World>, With<Handle<Scene>>)>,
    mut models_query: Query<
        (&rsm::Model, &mut AnimationPlayer),
        (Added<rsm::Model>, Added<AnimationPlayer>),
    >,
) {
    let Ok(_) = worlds_query.get_single() else {
        bevy::log::error!("There are 0 or more than 1 Worlds.");
        return;
    };

    for (model, mut animation_player) in models_query.iter_mut() {
        animation_player.play(model.animation.clone()).repeat();
    }
}
