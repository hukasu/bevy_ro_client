mod components;
#[cfg(feature = "debug")]
mod debug;
mod events;
mod loader;
mod materials;

use std::{collections::HashMap, time::Duration};

use bevy::{
    app::Plugin as BevyPlugin,
    asset::{AssetApp, Assets},
    pbr::MeshMaterial3d,
    prelude::{
        AnimationPlayer, AnimationTransitions, ChildOf, Children, Commands, Entity, Name, Query,
        ResMut, Transform, Trigger, With,
    },
};
use materials::RsmMaterial;

pub use self::{components::Model, events::StartPropAnimation, loader::AssetLoader};

use super::rsw::{AnimatedProp, WorldLoaded};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<components::Model>()
            // Loader
            .register_asset_loader(AssetLoader)
            // Observers
            .add_observer(start_rsm)
            .add_observer(invert_materials)
            .add_observer(start_rsm_animation);

        // Materials
        app.add_plugins(materials::Plugin);

        #[cfg(feature = "debug")]
        app.add_plugins(debug::Plugin);
    }
}

fn start_rsm(
    trigger: Trigger<WorldLoaded>,
    mut commands: Commands,
    children: Query<&Children>,
    animated_props: Query<(&Children, &AnimatedProp)>,
) {
    let Ok(world_children) = children.get(trigger.target()) else {
        bevy::log::error!("Just loaded world had no children.");
        return;
    };
    let Some(models_container) = world_children.iter().find_map(|child| {
        children
            .get(*child)
            .ok()
            .filter(|container| animated_props.contains(container[0]))
    }) else {
        bevy::log::error!("World does not have a container with AnimatedProps.");
        return;
    };

    for (animated_prop_children, animation_properties) in animated_props.iter_many(models_container)
    {
        if animated_prop_children.is_empty() {
            continue;
        }
        commands.trigger_targets(
            StartPropAnimation {
                speed: animation_properties.animation_speed,
                mode: animation_properties.animation_type,
            },
            animated_prop_children.to_vec(),
        )
    }
}

fn invert_materials(
    trigger: Trigger<WorldLoaded>,
    mut commands: Commands,
    children: Query<&Children>,
    animated_props: Query<(Entity, &Transform), With<AnimatedProp>>,
    materials: Query<&MeshMaterial3d<RsmMaterial>>,
    mut rsm_materials: ResMut<Assets<RsmMaterial>>,
) {
    let Ok(world_children) = children.get(trigger.target()) else {
        bevy::log::error!("Just loaded world had no children.");
        return;
    };
    let Some(models_container) = world_children.iter().find_map(|child| {
        children
            .get(*child)
            .ok()
            .filter(|container| animated_props.contains(container[0]))
    }) else {
        bevy::log::error!("World does not have a container with AnimatedProps.");
        return;
    };

    let mut new_material_cache = HashMap::new();
    for (prop, _) in animated_props
        .iter_many(models_container)
        .filter(|(_, transform)| {
            (transform.scale.x * transform.scale.y * transform.scale.z).is_sign_negative()
        })
    {
        for child_of_inverted_model in children.iter_descendants(prop) {
            if let Ok(material) = materials.get(child_of_inverted_model) {
                let new_handle = new_material_cache
                    .entry(material.0.id())
                    .or_insert_with(|| {
                        if let Some(mut clone) = rsm_materials.get(material.id()).cloned() {
                            clone.inverse_scale = true;
                            rsm_materials.add(clone)
                        } else {
                            material.0.clone()
                        }
                    });
                commands
                    .entity(child_of_inverted_model)
                    .insert(MeshMaterial3d(new_handle.clone()));
            }
        }
    }
}

fn start_rsm_animation(
    trigger: Trigger<StartPropAnimation>,
    models: Query<(Entity, &ChildOf, &Model)>,
    mut animation_graphs: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    names: Query<&Name>,
) {
    let Ok((entity, child, model)) = models.get(trigger.target()) else {
        bevy::log::trace!(
            "Prop {} is missing one or more of the required animation components.",
            trigger.target()
        );
        return;
    };

    let Ok((mut animation_player, mut animation_trasition)) = animation_graphs.get_mut(entity)
    else {
        return;
    };

    let name = names
        .get(child.parent())
        .map(|name| name.as_str())
        .unwrap_or("Unnamed AnimatedProp");
    bevy::log::trace!("Starting animation of prop {}", name);

    let animation_properties = trigger.event();

    if matches!(animation_properties.mode, 0) || model.animation.is_none() {
        return;
    }

    let Some(model_animation) = &model.animation else {
        unreachable!()
    };

    let animation = animation_trasition
        .play(
            &mut animation_player,
            model_animation.animation_node_index,
            Duration::default(),
        )
        .set_speed(animation_properties.speed);

    if matches!(animation_properties.mode, 2) {
        animation.repeat();
    }
}
