use bevy::{
    asset::Handle,
    image::Image,
    prelude::{ReflectResource, Resource},
    reflect::Reflect,
};

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct LoadingScreens {
    pub loading_screens: Vec<Handle<Image>>,
}
