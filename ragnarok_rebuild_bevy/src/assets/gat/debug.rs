use bevy::{
    app::PostUpdate,
    color::palettes,
    math::Vec3,
    prelude::{
        Gizmos, GlobalTransform, IntoSystemConfigs, Query, ReflectResource, Res, Resource,
        TransformSystem,
    },
    reflect::Reflect,
    render::{primitives::Aabb, view::VisibilitySystems},
};

use crate::helper::AabbExt;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Resources
            .register_type::<GatDebug>()
            .init_resource::<GatDebug>()
            // Systems
            .add_systems(
                PostUpdate,
                (show_hovered.run_if(show_hovered_condition),)
                    .after(VisibilitySystems::CheckVisibility)
                    .after(TransformSystem::TransformPropagate),
            );
    }
}

#[derive(Debug, Clone, Default, Resource, Reflect)]
#[reflect(Resource)]
pub struct GatDebug {
    show_hovered: bool,
}

fn show_hovered(
    mut gizmos: Gizmos,
    hovered_tile: Res<super::resources::HoveredTile>,
    gats: Query<(&super::components::Tiles, &GlobalTransform)>,
) {
    let Ok((gat, gat_transform)) = gats.get_single() else {
        return;
    };
    let Some(hovered) = **hovered_tile else {
        return;
    };

    let x = hovered.x as f32 - (gat.width / 2) as f32;
    let z = hovered.z as f32 - (gat.height / 2) as f32;

    // Hovered tile is in game space but x and z are in world scape, so we scale
    // only Y by the transform
    let Some(aabb) = Aabb::enclosing([
        Vec3::new(x, hovered.tile.bottom_left, z),
        Vec3::new(x + 1., hovered.tile.bottom_right, z),
        Vec3::new(x, hovered.tile.top_left, z + 1.),
        Vec3::new(x + 1., hovered.tile.top_right, z + 1.),
    ]) else {
        unreachable!("List of points passed to Aabb::enclosing will never be empty.")
    };

    let transform = gat_transform.compute_transform();
    gizmos.cuboid(
        aabb.compute_global_transform(GlobalTransform::from(
            transform.with_scale(transform.scale.with_x(1.).with_z(1.)),
        )),
        palettes::tailwind::EMERALD_400,
    );
}

fn show_hovered_condition(
    gat_debug: Res<GatDebug>,
    hovered_tile: Res<super::resources::HoveredTile>,
) -> bool {
    gat_debug.show_hovered & hovered_tile.is_some()
}
