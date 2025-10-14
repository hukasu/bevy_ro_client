use bevy_app::Update;
use bevy_asset::AssetServer;
use bevy_camera::primitives::Aabb;
use bevy_color::{Color, palettes};
use bevy_ecs::{
    entity::Entity,
    event::Event,
    observer::On,
    query::With,
    reflect::ReflectResource,
    resource::Resource,
    schedule::{IntoScheduleConfigs, common_conditions::resource_changed},
    system::{Commands, Local, Query, Res, ResMut},
};
use bevy_gizmos::{GizmoAsset, aabb::ShowAabbGizmo, retained::Gizmo};
use bevy_log::debug;
use bevy_math::Vec3;
use bevy_reflect::Reflect;

use crate::Tile;

pub(crate) struct Plugin;

impl bevy_app::Plugin for Plugin {
    fn build(&self, app: &mut bevy_app::App) {
        // Resources
        app.register_type::<GatDebug>().init_resource::<GatDebug>();
        app.add_systems(
            Update,
            trigger_on_changes.run_if(resource_changed::<GatDebug>),
        );
        // Observers
        app.add_observer(toggle_gat_aabbs);
        app.add_observer(toggle_gat_quads);
    }
}

#[derive(Debug, Clone, Copy, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GatDebug {
    show_aabbs: bool,
    show_quads: bool,
}

#[derive(Debug, Event)]
pub struct ToggleGatAabbs;

#[derive(Debug, Event)]
pub struct ToggleGatQuads;

fn toggle_gat_aabbs(_event: On<ToggleGatAabbs>, mut gat_debug: ResMut<GatDebug>) {
    debug!("Toggling Gat Aabbs");
    gat_debug.show_aabbs = !gat_debug.show_aabbs;
}

fn enable_gat_aabbs(mut commands: Commands, tiles: Query<Entity, With<Tile>>) {
    debug!("Enabling Gat Aabbs");
    let tile_aabb_color = palettes::tailwind::YELLOW_500.into();
    for tile in tiles {
        commands.entity(tile).insert(ShowAabbGizmo {
            color: Some(tile_aabb_color),
        });
    }
}

fn disable_gat_aabbs(mut commands: Commands, tiles: Query<Entity, With<Tile>>) {
    debug!("Disabling Gat Aabbs");
    for tile in tiles {
        commands.entity(tile).remove::<ShowAabbGizmo>();
    }
}

fn toggle_gat_quads(_event: On<ToggleGatQuads>, mut gat_debug: ResMut<GatDebug>) {
    debug!("Toggling Gat Quads");
    gat_debug.show_quads = !gat_debug.show_quads;
}

fn enable_gat_quads(
    mut commands: Commands,
    tiles: Query<(Entity, &Tile, &Aabb)>,
    asset_server: Res<AssetServer>,
) {
    debug!("Enabling Gat Quads");
    let tile_quad_color: Color = palettes::tailwind::ORANGE_500.into();
    for (entity, tile, aabb) in tiles {
        let mut gizmo = GizmoAsset::new();
        let max_y = tile
            .bottom_left
            .max(tile.bottom_right)
            .max(tile.top_left)
            .max(tile.top_right);
        let min = aabb.min();
        let max = aabb.max();
        gizmo.line(
            Vec3::new(max.x, tile.bottom_right - max_y, max.z),
            Vec3::new(max.x, tile.top_right - max_y, min.z),
            tile_quad_color,
        );
        gizmo.line(
            Vec3::new(max.x, tile.top_right - max_y, min.z),
            Vec3::new(min.x, tile.top_left - max_y, min.z),
            tile_quad_color,
        );
        gizmo.line(
            Vec3::new(min.x, tile.top_left - max_y, min.z),
            Vec3::new(min.x, tile.bottom_left - max_y, max.z),
            tile_quad_color,
        );
        gizmo.line(
            Vec3::new(min.x, tile.bottom_left - max_y, max.z),
            Vec3::new(max.x, tile.bottom_right - max_y, max.z),
            tile_quad_color,
        );
        commands.entity(entity).insert(Gizmo {
            handle: asset_server.add(gizmo),
            ..Default::default()
        });
    }
}

fn disable_gat_quads(mut commands: Commands, tiles: Query<Entity, With<Tile>>) {
    debug!("Disabling Gat Quads");
    for tile in tiles {
        commands.entity(tile).remove::<Gizmo>();
    }
}

fn trigger_on_changes(
    mut commands: Commands,
    gat_debug: Res<GatDebug>,
    mut gat_debug_cache: Local<GatDebug>,
) {
    if gat_debug.show_aabbs != gat_debug_cache.show_aabbs {
        match gat_debug.show_aabbs {
            true => commands.run_system_cached(enable_gat_aabbs),
            false => commands.run_system_cached(disable_gat_aabbs),
        }
    }
    if gat_debug.show_quads != gat_debug_cache.show_quads {
        match gat_debug.show_quads {
            true => commands.run_system_cached(enable_gat_quads),
            false => commands.run_system_cached(disable_gat_quads),
        }
    }

    *gat_debug_cache = *gat_debug;
}
