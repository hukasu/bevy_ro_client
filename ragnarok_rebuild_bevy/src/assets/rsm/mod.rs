mod components;
mod events;
mod loader;
mod materials;

use std::time::Duration;

use bevy::{
    app::Plugin as BevyPlugin,
    asset::AssetApp,
    core::Name,
    prelude::{AnimationPlayer, AnimationTransitions, Entity, Parent, Query, Trigger},
};

pub use self::{components::Model, events::StartPropAnimation, loader::AssetLoader};

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Types
            .register_type::<components::Model>()
            // Loader
            .register_asset_loader(AssetLoader)
            // Observers
            .observe(start_rsm_animation);

        // Materials
        app.add_plugins(materials::Plugin);
    }
}

fn start_rsm_animation(
    trigger: Trigger<StartPropAnimation>,
    models: Query<(Entity, &Parent, &Model)>,
    mut animation_graphs: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
    names: Query<&Name>,
) {
    let Ok((entity, parent, model)) = models.get(trigger.entity()) else {
        bevy::log::trace!(
            "Prop {} is missing one or more of the required animation components.",
            trigger.entity()
        );
        return;
    };

    let Ok((mut animation_player, mut animation_trasition)) = animation_graphs.get_mut(entity)
    else {
        return;
    };

    let name = names
        .get(parent.get())
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
