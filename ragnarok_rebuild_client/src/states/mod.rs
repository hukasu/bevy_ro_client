mod game;

use std::time::Duration;

use bevy::{
    pbr::MeshMaterial3d,
    prelude::{
        AnimationPlayer, AnimationTransitions, AppExtStates, ChildOf, Children, Entity, Name,
        OnEnter, Query, Transform,
    },
};

use ragnarok_rebuild_bevy::assets::rsw::AnimatedProp;
use ragnarok_rsm::{
    components::{Model, ModelInvertedMaterial},
    materials::RsmMaterial,
};

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
