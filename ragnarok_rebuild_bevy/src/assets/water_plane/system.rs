use bevy::{
    asset::{Assets, Handle},
    ecs::system::{Query, Res, ResMut},
    pbr::StandardMaterial,
    time::Time,
};

use super::WaterPlane;

pub fn update_texture(
    mut query: Query<(&mut WaterPlane, &Handle<StandardMaterial>)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    timer: Res<Time>,
) {
    for (mut plane, material_handle) in query.iter_mut() {
        plane.timer.tick(timer.delta());
        if plane.timer.just_finished() {
            plane.current_frame += 1;
            plane.current_frame %= 32;

            let Some(material) = material_assets.get_mut(material_handle) else {
                bevy::log::error!("Water Plane has a StandardMaterial that does not exist.");
                continue;
            };

            material.base_color_texture = Some(plane.frames[plane.current_frame].clone());
        }
    }
}
