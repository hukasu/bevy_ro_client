use bevy::{
    app::Startup,
    audio::SpatialListener,
    pbr::{Atmosphere, AtmosphereSettings},
    post_process::bloom::Bloom,
    prelude::{Camera, Camera3d, Commands},
    render::view::Hdr,
};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // Systems
            .add_systems(Startup, create_camera);
    }
}

fn create_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Camera {
            ..Default::default()
        },
        Hdr,
        Atmosphere::EARTH,
        AtmosphereSettings {
            scene_units_to_m: 5.,
            ..Default::default()
        },
        Bloom::NATURAL,
        SpatialListener::default(),
    ));
}
