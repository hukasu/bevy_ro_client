use bevy::{
    prelude::{Deref, DerefMut, ReflectResource, Resource},
    reflect::Reflect,
};

#[derive(Debug, Default, Deref, DerefMut, Resource, Reflect)]
#[reflect(Resource)]
pub struct HoveredTile(Option<super::components::TileRef>);
