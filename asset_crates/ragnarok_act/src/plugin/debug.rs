use bevy_app::Update;
use bevy_asset::AssetServer;
use bevy_color::palettes;
use bevy_ecs::{
    entity::Entity,
    prelude::resource_exists_and_changed,
    query::With,
    reflect::ReflectResource,
    resource::Resource,
    schedule::IntoScheduleConfigs,
    system::{Commands, Local, Query, Res},
};
use bevy_gizmos::{GizmoAsset, retained::Gizmo};
use bevy_math::Isometry3d;
use bevy_reflect::{Reflect, prelude::ReflectDefault};

use crate::components::{ActorAnchor, ActorLayer};

pub struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        app
            // Resources
            .register_type::<ActDebug>()
            .init_resource::<ActDebug>()
            // Systems
            .add_systems(
                Update,
                (toggle_show_layers, toggle_show_anchors)
                    .run_if(resource_exists_and_changed::<ActDebug>),
            );
    }
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Default, Resource)]
pub struct ActDebug {
    pub show_layers: bool,
    pub show_anchors: bool,
}

fn toggle_show_layers(
    mut commands: Commands,
    layers: Query<Entity, With<ActorLayer>>,
    debug: Res<ActDebug>,
    asset_server: Res<AssetServer>,
    mut local: Local<bool>,
) {
    if debug.show_layers == *local {
        return;
    }
    *local = debug.show_layers;

    let entities = Vec::from_iter(layers.iter());
    if debug.show_layers {
        log::info!("Enabling Actor layer gizmos.");
        let mut gizmo = GizmoAsset::new();
        gizmo.sphere(Isometry3d::default(), 1., palettes::tailwind::RED_400);
        let gizmos = asset_server.add(gizmo);
        commands.insert_batch(entities.into_iter().map(move |entity| {
            (
                entity,
                Gizmo {
                    handle: gizmos.clone(),
                    ..Default::default()
                },
            )
        }));
    } else {
        log::info!("Disabling Actor layer gizmos.");
        for entity in entities {
            commands.entity(entity).remove::<Gizmo>();
        }
    }
}

fn toggle_show_anchors(
    mut commands: Commands,
    layers: Query<Entity, With<ActorAnchor>>,
    debug: Res<ActDebug>,
    asset_server: Res<AssetServer>,
    mut local: Local<bool>,
) {
    if debug.show_anchors == *local {
        return;
    }
    *local = debug.show_anchors;

    let entities = Vec::from_iter(layers.iter());
    if debug.show_anchors {
        log::info!("Enabling Actor anchor gizmos.");
        let mut gizmo = GizmoAsset::new();
        gizmo.sphere(Isometry3d::default(), 1., palettes::tailwind::GREEN_900);
        let gizmos = asset_server.add(gizmo);
        commands.insert_batch(entities.into_iter().map(move |entity| {
            (
                entity,
                Gizmo {
                    handle: gizmos.clone(),
                    ..Default::default()
                },
            )
        }));
    } else {
        log::info!("Disabling Actor anchor gizmos.");
        for entity in entities {
            commands.entity(entity).remove::<Gizmo>();
        }
    }
}
