use bevy_asset::Handle;
use bevy_ecs::{component::Component, reflect::ReflectComponent};
use bevy_image::Image;
use bevy_reflect::Reflect;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
/// A palette containing 256 colors in a 1D image
pub struct Palette(pub Handle<Image>);
