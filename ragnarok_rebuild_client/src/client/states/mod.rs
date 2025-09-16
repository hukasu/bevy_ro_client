mod game;

use bevy::{
    pbr::MeshMaterial3d,
    prelude::{
        AnimationPlayer, AppExtStates, ChildOf, Children, Entity, Name, OnEnter, Query, Transform,
    },
};

use bevy_ragnarok_rsm::{materials::RsmMaterial, Model, ModelInvertedMaterial};
use ragnarok_rsw::components::AnimatedProp;

pub use self::game::GameState;
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_state::<GameState>().add_systems(
            OnEnter(GameState::Game),
            (start_animations, invert_rsm_materials),
        );
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

fn invert_rsm_materials(
    models: Query<(Entity, &ChildOf)>,
    transforms: Query<&Transform>,
    children: Query<&Children>,
    mut materials: Query<(&mut MeshMaterial3d<RsmMaterial>, &mut ModelInvertedMaterial)>,
) {
    let inverted_models = models.iter().filter_map(|(entity, parent)| {
        #[expect(clippy::unwrap_used, reason = "Relationship must be valid")]
        let transform = transforms.get(parent.parent()).unwrap();
        if transform.scale.x.signum() * transform.scale.y.signum() * transform.scale.z.signum() < 0.
        {
            Some(entity)
        } else {
            None
        }
    });
    for inverted in inverted_models {
        for child in children.iter_descendants(inverted) {
            if let Ok((mut rsm, mut inverted)) = materials.get_mut(child) {
                std::mem::swap(&mut rsm.0, &mut inverted.0);
            }
        }
    }
}
