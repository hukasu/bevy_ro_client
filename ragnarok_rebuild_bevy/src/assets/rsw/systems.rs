use bevy::{
    animation::AnimationPlayer,
    asset::Handle,
    core::Name,
    ecs::{
        entity::Entity,
        query::{Added, With},
        system::Query,
    },
    hierarchy::{Children, Parent},
    scene::Scene,
};

use crate::assets::rsm;

pub fn start_animations(
    worlds_query: Query<&Children, (With<super::World>, With<Handle<Scene>>)>,
    world_model_containers: Query<&Children, With<super::components::Models>>,
    world_models: Query<(&Name, &super::components::WorldModel, &Children)>,
    mut models_query: Query<
        (&rsm::Model, &mut AnimationPlayer),
        (Added<rsm::Model>, Added<AnimationPlayer>),
    >,
) {
    if models_query.is_empty() {
        return;
    }

    let Ok(children_of_world) = worlds_query.get_single() else {
        bevy::log::error!("There are 0 or more than 1 Worlds.");
        return;
    };

    let world_models_container = world_model_containers
        .iter_many(children_of_world)
        .collect::<Vec<_>>();
    let [children_of_models_container] = world_models_container.as_slice() else {
        bevy::log::error!("World does not have a Models container or has multiple.");
        return;
    };

    for (name, model, children) in world_models.iter_many(*children_of_models_container) {
        let children_of_model = children.iter().collect::<Vec<_>>();
        let [single_child] = children_of_model.as_slice() else {
            bevy::log::error!("Model {name} has more than one root.");
            return;
        };

        let Ok((model_animation, mut animation_player)) = models_query.get_mut(**single_child)
        else {
            bevy::log::error!("Child of Model {name} does not have AnimationPlayer.");
            return;
        };
        animation_player
            .play(model_animation.animation.clone())
            .set_speed(model.animation_speed)
            .repeat();
    }
}
