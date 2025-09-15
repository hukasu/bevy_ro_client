use bevy_app::Update;
use bevy_asset::AssetServer;
use bevy_color::palettes;
use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    reflect::ReflectResource,
    resource::Resource,
    schedule::{IntoScheduleConfigs, common_conditions::not},
    system::{Commands, Populated, Res},
};
use bevy_gizmos::{GizmoAsset, retained::Gizmo};
use bevy_math::Isometry3d;
use bevy_reflect::{Reflect, prelude::ReflectDefault};
use log::info;

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
                (
                    add_gizmos_to_layers.run_if(showing_layers),
                    remove_gizmos_from_layers.run_if(not(showing_layers)),
                    add_gizmos_to_anchors.run_if(showing_anchors),
                    remove_gizmos_from_anchors.run_if(not(showing_anchors)),
                ),
            );
    }
}

#[derive(Debug, Default, Resource, Reflect)]
#[reflect(Default, Resource)]
pub struct ActDebug {
    pub show_layers: bool,
    pub show_anchors: bool,
}

fn showing_layers(debug: Res<ActDebug>) -> bool {
    debug.show_layers
}

fn add_gizmos_to_layers(
    mut commands: Commands,
    layers: Populated<Entity, (With<ActorLayer>, Without<Gizmo>)>,
    asset_server: Res<AssetServer>,
) {
    info!("Enabling gizmos for {} Actor layers.", layers.iter().len());
    let mut gizmo = GizmoAsset::new();
    gizmo.sphere(Isometry3d::default(), 1., palettes::tailwind::RED_400);
    let gizmos = asset_server.add(gizmo);
    for layer in layers {
        commands.entity(layer).insert(Gizmo {
            handle: gizmos.clone(),
            ..Default::default()
        });
    }
}

fn remove_gizmos_from_layers(
    mut commands: Commands,
    layers: Populated<Entity, (With<ActorLayer>, With<Gizmo>)>,
) {
    info!("Removing gizmos for {} Actor layers.", layers.iter().len());
    for layer in layers {
        commands.entity(layer).remove::<Gizmo>();
    }
}

fn showing_anchors(debug: Res<ActDebug>) -> bool {
    debug.show_anchors
}

fn add_gizmos_to_anchors(
    mut commands: Commands,
    anchors: Populated<Entity, (With<ActorLayer>, Without<Gizmo>)>,
    asset_server: Res<AssetServer>,
) {
    info!("Enabling gizmos for {} Actor anchor.", anchors.iter().len());
    let mut gizmo = GizmoAsset::new();
    gizmo.sphere(Isometry3d::default(), 1., palettes::tailwind::GREEN_900);
    let gizmos = asset_server.add(gizmo);
    for anchor in anchors {
        commands.entity(anchor).insert(Gizmo {
            handle: gizmos.clone(),
            ..Default::default()
        });
    }
}

fn remove_gizmos_from_anchors(
    mut commands: Commands,
    anchors: Populated<Entity, (With<ActorAnchor>, With<Gizmo>)>,
) {
    info!("Removing gizmos for {} Actor layers.", anchors.iter().len());
    for anchor in anchors {
        commands.entity(anchor).remove::<Gizmo>();
    }
}
