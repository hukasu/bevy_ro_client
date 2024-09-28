use std::fmt::Display;

use bevy::{
    asset::{io::Reader, AsyncReadExt, LoadContext},
    color::Color,
    math::{IVec2, Vec2},
    utils::ConditionalSendFuture,
};

use ragnarok_rebuild_assets::act;

use crate::assets::paths;

use super::{
    assets::AnimationLayerSprite, Animation, AnimationClip, AnimationEvent, AnimationFrame,
    AnimationLayer,
};

const ANIMATION_FRAME_TIME_FACTOR: f32 = 24. / 1000.;

pub struct AssetLoader;

impl bevy::asset::AssetLoader for AssetLoader {
    type Asset = Animation;
    type Settings = ();
    type Error = AssetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let actor = act::Act::from_reader(&mut data.as_slice())?;

            Self::generate_actor(load_context, &actor)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["act"]
    }
}

impl AssetLoader {
    fn generate_actor(
        load_context: &mut LoadContext,
        act: &act::Act,
    ) -> Result<Animation, AssetLoaderError> {
        Ok(Animation {
            clips: Self::generate_clips(load_context, act)?,
        })
    }

    fn generate_clips(
        load_context: &mut LoadContext,
        act: &act::Act,
    ) -> Result<Box<[AnimationClip]>, AssetLoaderError> {
        act.animation_clips
            .iter()
            .zip(&act.frame_times)
            .enumerate()
            .map(|(i, (clip, frame_time))| {
                Ok(AnimationClip {
                    frame_time: frame_time * ANIMATION_FRAME_TIME_FACTOR,
                    frames: Self::generate_frames(load_context, clip, &act.animation_events, i)?,
                })
            })
            .collect::<Result<Box<[_]>, _>>()
    }

    fn generate_frames(
        load_context: &mut LoadContext,
        clip: &act::AnimationClip,
        events: &[act::AnimationEvent],
        clip_id: usize,
    ) -> Result<Box<[AnimationFrame]>, AssetLoaderError> {
        clip.animation_frames
            .iter()
            .enumerate()
            .map(|(i, frame)| {
                let ani_event = if frame.animation_event_id == -1 {
                    None
                } else {
                    let Ok(event_id) = usize::try_from(frame.animation_event_id.unsigned_abs()) else {
                        bevy::log::error!("Act has an event id ({}) that does not fit on a usize on Clip{}/Frame{}.", frame.animation_event_id, clip_id, i);
                        return Err(AssetLoaderError::UsizeConversion);
                    };
                    match events.get(event_id) {
                        Some(event) => {
                            match event.name.as_ref() {
                                "atk" => {
                                    Some(AnimationEvent::Attack)
                                }
                                sound => {
                                    Some(AnimationEvent::Sound(load_context.load(format!("{}{}", paths::WAV_FILES_FOLDER, sound))))
                                }
                            }
                        }
                        None => {
                            bevy::log::error!("Actor {:?} has an event that accesses out of bounds on Clip{}/Frame{}.", load_context.path(), clip_id, i);
                            None
                        }
                    }
                };

                Ok(AnimationFrame {
                                    layers: Self::generate_layers(frame)?,
                                    anchors: Box::new([]),
                                    event: ani_event,
                                })
            })
            .collect::<Result<Box<[_]>, _>>()
    }

    fn generate_layers(
        frame: &act::AnimationFrame,
    ) -> Result<Box<[AnimationLayer]>, AssetLoaderError> {
        frame
            .sprite_layers
            .iter()
            .map(|layer| {
                let origin = IVec2::new(layer.position_u, layer.position_v);
                let sprite = match layer.image_type_id {
                    0 => AnimationLayerSprite::Indexed(
                        usize::try_from(layer.spritesheet_cell_index)
                            .map_err(|_| AssetLoaderError::UsizeConversion)?,
                    ),
                    1 => AnimationLayerSprite::TrueColor(
                        usize::try_from(layer.spritesheet_cell_index)
                            .map_err(|_| AssetLoaderError::UsizeConversion)?,
                    ),
                    _ => unreachable!(
                        "Act file should not be loaded if it has a value different from this"
                    ),
                };
                let is_flipped = layer.is_flipped_v;
                let tint = Color::srgba_u8(
                    layer.tint.red,
                    layer.tint.green,
                    layer.tint.blue,
                    layer.tint.alpha,
                );
                let scale = Vec2::new(layer.scale_u, layer.scale_v);
                let rotation = (layer.rotation as f32).to_radians();
                let image_size = IVec2::new(layer.image_width, layer.image_height);

                Ok(AnimationLayer {
                    origin,
                    sprite,
                    is_flipped,
                    tint,
                    scale,
                    rotation,
                    image_size,
                })
            })
            .collect::<Result<Box<[_]>, AssetLoaderError>>()
    }
}

#[derive(Debug)]
pub enum AssetLoaderError {
    Act(act::Error),
    Io(std::io::Error),
    UsizeConversion,
}

impl Display for AssetLoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetLoaderError::Act(err) => write!(f, "Act failed loading with error '{}'.", err),
            AssetLoaderError::Io(err) => write!(
                f,
                "AssetLoader failed to read Act contents with error '{}'.",
                err
            ),
            AssetLoaderError::UsizeConversion => {
                write!(f, "AssetLoader failed to convert number into usize.")
            }
        }
    }
}

impl std::error::Error for AssetLoaderError {}

impl From<act::Error> for AssetLoaderError {
    fn from(value: act::Error) -> Self {
        Self::Act(value)
    }
}

impl From<std::io::Error> for AssetLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
