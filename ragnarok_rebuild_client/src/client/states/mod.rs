mod game;

use bevy::prelude::{AnimationPlayer, AppExtStates, Children, Name, OnEnter, Query};

use bevy_ragnarok_rsm::Model;
use bevy_ragnarok_rsw::AnimatedProp;

pub use self::game::GameState;
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<GameState>()
            .add_systems(OnEnter(GameState::Game), start_animations);
    }
}

fn start_animations(
    animated_props: Query<(&Name, &AnimatedProp, &Children)>,
    mut models: Query<(&Model, &mut AnimationPlayer)>,
) {
    for (name, prop, children) in animated_props {
        for child in children {
            if let Ok((model, mut player)) = models.get_mut(*child) {
                let Some(animation) = &model.animation else {
                    continue;
                };

                match prop.animation_type {
                    0 => (),
                    1 => {
                        bevy::log::trace!("Starting animation of {}.", name);
                        player
                            .play(animation.animation_node_index)
                            .set_speed(prop.animation_speed);
                    }
                    2 => {
                        bevy::log::trace!("Starting repeating animation of {}.", name);
                        player
                            .play(animation.animation_node_index)
                            .set_speed(prop.animation_speed)
                            .repeat();
                    }
                    _ => unreachable!("Invalid animation type {}.", prop.animation_type),
                }
            } else {
                bevy::log::warn!("Child of {} was not a prop.", name);
            }
        }
    }
}
