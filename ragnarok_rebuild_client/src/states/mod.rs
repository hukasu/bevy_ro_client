mod game;

use std::time::Duration;

use bevy::prelude::{
    AnimationPlayer, AnimationTransitions, AppExtStates, ChildOf, Name, OnEnter, Query,
};

use ragnarok_rebuild_bevy::assets::rsw::AnimatedProp;
use ragnarok_rsm::components::Model;

pub use self::game::GameState;
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Game), start_animations);
    }
}

fn start_animations(
    models: Query<(
        &ChildOf,
        &Model,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    animated_props: Query<(&Name, &AnimatedProp)>,
) {
    for (prop, model, mut player, mut transitions) in models {
        let Ok((rsm_name, animated_prop)) = animated_props.get(prop.parent()) else {
            bevy::log::warn!("{} did not have `AnimatedProp`.", prop.parent());
            continue;
        };

        let Some(animation) = &model.animation else {
            continue;
        };

        match animated_prop.animation_type {
            0 => (),
            1 => {
                bevy::log::trace!("Starting animation of {}.", rsm_name);
                transitions.play(
                    &mut player,
                    animation.animation_node_index,
                    Duration::default(),
                );
            }
            2 => {
                bevy::log::trace!("Starting repeating animation of {}.", rsm_name);
                transitions
                    .play(
                        &mut player,
                        animation.animation_node_index,
                        Duration::default(),
                    )
                    .repeat();
            }
            _ => unreachable!("Invalid animation type {}.", animated_prop.animation_type),
        }
    }
}
