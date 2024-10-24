use bevy::{
    math::{bounding::RayCast3d, Dir3A, Vec3A},
    prelude::{Deref, DerefMut, ReflectResource, Resource},
    reflect::Reflect,
};

#[derive(Debug, Deref, DerefMut, Resource, Reflect)]
#[reflect(Resource)]
pub struct TileRayCast(pub RayCast3d);

impl Default for TileRayCast {
    fn default() -> Self {
        TileRayCast(RayCast3d::new(Vec3A::ZERO, Dir3A::Z, 0.))
    }
}

#[derive(Debug, Default, Deref, DerefMut, Resource, Reflect)]
#[reflect(Resource)]
pub struct HoveredTile(Option<super::components::TileRef>);
