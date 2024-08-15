mod components;
mod loader;

use std::time::Duration;

use bevy::{
    app::Plugin as BevyPlugin,
    asset::AssetApp,
    prelude::{AnimationPlayer, AnimationTransitions, OnAdd, Query, Trigger},
};

pub use self::{components::Model, loader::AssetLoader};

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
    }
}

pub fn start_rsm_animation(
    trigger: Trigger<OnAdd, Model>,
    mut animation_graphs: Query<(&Model, &mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    let Ok((model, mut animation_player, mut animation_trasition)) =
        animation_graphs.get_mut(trigger.entity())
    else {
        return;
    };

    animation_trasition
        .play(
            &mut animation_player,
            model.animation_node_index,
            Duration::default(),
        )
        .repeat();
}
