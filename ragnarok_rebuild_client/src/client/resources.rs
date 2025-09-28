use bevy::{
    asset::Handle,
    prelude::{ReflectResource, Resource},
    reflect::Reflect,
};

use bevy_ragnarok_rsw::assets::RswWorld;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct LoadingWorld {
    pub world: Handle<RswWorld>,
}
