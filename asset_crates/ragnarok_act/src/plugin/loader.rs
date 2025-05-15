use std::{
    fmt::{Debug, Display},
    num::TryFromIntError,
    path::PathBuf,
};

use bevy_animation::{
    AnimationClip, AnimationPlayer, AnimationTarget, AnimationTargetId, animated_field,
    gltf_curves::SteppedKeyframeCurve,
    graph::{AnimationGraph, AnimationGraphHandle},
    prelude::{AnimatableCurve, AnimatableProperty, AnimatedField, AnimationTransitions},
};
use bevy_asset::{Handle, LoadContext, io::Reader};
use bevy_color::{Color, LinearRgba};
use bevy_ecs::{
    hierarchy::Children,
    name::Name,
    spawn::{SpawnIter, SpawnRelated},
    world::World,
};
use bevy_math::{Quat, Vec3};
use bevy_platform::collections::HashMap;
use bevy_reflect::{FromReflect, GetTypeRegistration, Typed};
use bevy_render::{mesh::Mesh3d, view::Visibility};
use bevy_scene::Scene;
use bevy_transform::components::Transform;

use crate::{
    Act, AnimationEvent,
    assets::ActorAnimations,
    components::{ActorAnchor, ActorLayer, ActorPlayer, SpritesheetIndex},
    events::ActorSound,
};

use super::IDENTITY_PLANE_HANDLE;

const ANIMATION_FRAME_TIME_FACTOR: f32 = 24. / 1000.;
const ACTOR_SCALE_FACTOR: f32 = 5. / 32.;

pub struct AssetLoader {
    pub audio_path_prefix: PathBuf,
}

impl bevy_asset::AssetLoader for AssetLoader {
    type Asset = ActorAnimations;
    type Settings = ();
    type Error = AssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;
        let actor = Act::from_reader(&mut data.as_slice())?;

        let (layers, anchors) =
            actor
                .animation_clips
                .iter()
                .fold((0, 0), |(layers, anchors), clip| {
                    let (clip_layers, clip_anchors) = clip.layers_and_anchors();
                    (layers.max(clip_layers), anchors.max(clip_anchors))
                });

        let animation_graph = self.generate_animation(&actor, layers, anchors, load_context)?;

        let scene = {
            let scene = self.generate_actor_scene(animation_graph.clone(), layers, anchors)?;
            load_context.add_labeled_asset("Scene".to_string(), scene)
        };

        Ok(ActorAnimations {
            animation_graph,
            scene,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["act"]
    }
}

impl AssetLoader {
    fn generate_actor_scene(
        &self,
        animation_graph: Handle<AnimationGraph>,
        max_layers: usize,
        max_anchors: usize,
    ) -> Result<Scene, AssetLoaderError> {
        let mut world = World::new();

        let root = world.spawn_empty().id();

        world.entity_mut(root).insert((
            ActorPlayer,
            Transform::from_scale(Vec3::splat(ACTOR_SCALE_FACTOR)),
            Visibility::default(),
            AnimationPlayer::default(),
            AnimationTransitions::default(),
            AnimationGraphHandle(animation_graph),
            Children::spawn((
                SpawnIter((0..max_layers).map(move |i| {
                    let name = Name::new(format!("Layer{i}"));
                    (
                        name.clone(),
                        ActorLayer::default(),
                        AnimationTarget {
                            id: AnimationTargetId::from_name(&name),
                            player: root,
                        },
                        Mesh3d(IDENTITY_PLANE_HANDLE),
                    )
                })),
                SpawnIter((0..max_anchors).map(move |i| {
                    let name = Name::new(format!("Anchor{i}"));
                    (
                        name.clone(),
                        ActorAnchor,
                        AnimationTarget {
                            id: AnimationTargetId::from_name(&name),
                            player: root,
                        },
                    )
                })),
            )),
        ));

        Ok(Scene::new(world))
    }

    fn generate_animation(
        &self,
        act: &Act,
        max_layers: usize,
        max_anchors: usize,
        load_context: &mut LoadContext,
    ) -> Result<Handle<AnimationGraph>, AssetLoaderError> {
        let mut animation_clips = Vec::new();

        for ((i, clip), frame_time) in act.animation_clips.iter().enumerate().zip(&act.frame_times)
        {
            let animation_clip = self.generate_clip(
                clip,
                &act.animation_events,
                max_layers,
                max_anchors,
                *frame_time,
                load_context,
            );

            animation_clips
                .push(load_context.add_labeled_asset(format!("Clip{}", i), animation_clip));
        }

        Ok(load_context.add_labeled_asset(
            "Animation".to_string(),
            AnimationGraph::from_clips(animation_clips).0,
        ))
    }

    fn generate_clip(
        &self,
        clip: &crate::AnimationClip,
        events: &[AnimationEvent],
        max_layers: usize,
        max_anchors: usize,
        frame_time: f32,
        load_context: &mut LoadContext,
    ) -> AnimationClip {
        let (layers, anchors) = clip.layers_and_anchors();
        assert!(layers <= max_layers);
        assert!(anchors <= max_anchors);

        let step = frame_time * ANIMATION_FRAME_TIME_FACTOR;

        let mut animation_clip = AnimationClip::default();

        let mut layer_clips = HashMap::new();
        layer_clips.extend((0..max_layers).map(|ani_target| (ani_target, LayerClip::default())));
        // let mut anchor_clips = HashMap::new();
        // anchor_clips.extend(
        //     (0..max_anchors)
        //         .map(|ani_target| (ani_target, Clip::default())),
        // );

        for (f, frame) in clip.animation_frames.iter().enumerate() {
            let frame_time = f as f32 * step;
            for (i, layer) in frame.sprite_layers.iter().enumerate() {
                let clip = unsafe { layer_clips.get_mut(&i).unwrap_unchecked() };
                clip.active.push((frame_time, true));
                clip.position.push((
                    frame_time,
                    Vec3::new(layer.position_u as f32, layer.position_v as f32, 0.),
                ));
                clip.rotation.push((
                    frame_time,
                    Quat::from_rotation_z((layer.rotation as f32).to_radians()),
                ));
                clip.scale
                    .push((frame_time, Vec3::new(layer.scale_u, layer.scale_v, 1.)));
                if layer.spritesheet_cell_index >= 0 {
                    let Ok(index) = usize::try_from(layer.spritesheet_cell_index) else {
                        unreachable!(
                            "Unaddresable spritesheet cell {}.",
                            layer.spritesheet_cell_index
                        );
                    };
                    let spritesheet = match layer.image_type_id {
                        0 => SpritesheetIndex::Indexed(index),
                        1 => SpritesheetIndex::TrueColor(index),
                        other => unreachable!("Invalid image type {}.", other),
                    };

                    clip.spritesheet_index.push((frame_time, spritesheet));
                }
                clip.uv_flip.push((frame_time, layer.is_flipped_v));
                clip.tint.push((
                    frame_time,
                    Color::srgba_u8(
                        layer.tint.red,
                        layer.tint.green,
                        layer.tint.blue,
                        layer.tint.alpha,
                    )
                    .into(),
                ));
            }

            for layer in frame.sprite_layers.len()..max_layers {
                let clip = unsafe { layer_clips.get_mut(&layer).unwrap_unchecked() };
                clip.active.push((frame_time, false))
            }

            if frame.animation_event_id >= 0 {
                if let Some(event) = usize::try_from(frame.animation_event_id)
                    .inspect_err(|_| {
                        log::error!("Can address animation event on current architecture.")
                    })
                    .ok()
                    .and_then(|id| events.get(id))
                {
                    match event.name.as_ref() {
                        "atk" => (),
                        sound => animation_clip.add_event(
                            frame_time,
                            ActorSound::new(
                                Name::new(sound.to_string()),
                                load_context.load(self.audio_path_prefix.join(sound)),
                            ),
                        ),
                    }
                }
            }
        }

        for (layer, layer_clip) in layer_clips {
            let name = Name::new(format!("Layer{layer}"));
            Self::push_stepped_curve(
                &name,
                &mut animation_clip,
                animated_field!(ActorLayer::active),
                layer_clip.active,
                step,
            );
            Self::push_stepped_curve(
                &name,
                &mut animation_clip,
                animated_field!(Transform::translation),
                layer_clip.position,
                step,
            );
            Self::push_stepped_curve(
                &name,
                &mut animation_clip,
                animated_field!(Transform::rotation),
                layer_clip.rotation,
                step,
            );
            Self::push_stepped_curve(
                &name,
                &mut animation_clip,
                animated_field!(Transform::scale),
                layer_clip.scale,
                step,
            );
            Self::push_stepped_curve(
                &name,
                &mut animation_clip,
                animated_field!(ActorLayer::spritesheet_index),
                layer_clip.spritesheet_index,
                step,
            );
            Self::push_stepped_curve(
                &name,
                &mut animation_clip,
                animated_field!(ActorLayer::uv_flip),
                layer_clip.uv_flip,
                step,
            );
            Self::push_stepped_curve(
                &name,
                &mut animation_clip,
                animated_field!(ActorLayer::tint),
                layer_clip.tint,
                step,
            );
        }

        animation_clip.set_duration(animation_clip.duration() + step);

        animation_clip
    }

    fn push_stepped_curve<
        T: Clone + Debug + Typed + GetTypeRegistration + FromReflect,
        P: Clone + AnimatableProperty<Property = T>,
    >(
        name: &Name,
        animation_clip: &mut AnimationClip,
        property: P,
        mut samples: Vec<(f32, T)>,
        step: f32,
    ) {
        if samples.len() == 1 {
            let head = samples[0].clone();
            if head.0 == 0. {
                samples.push((step, head.1));
            } else {
                samples.insert(0, (0., head.1));
            }
        }
        if !samples.is_empty() {
            if let Ok(curve) = SteppedKeyframeCurve::new(samples) {
                animation_clip.add_curve_to_target(
                    AnimationTargetId::from_name(name),
                    AnimatableCurve::new(property, curve),
                );
            } else {
                log::error!("Failed to build curve for {}.", name);
            }
        }
    }
}

#[derive(Default)]
struct LayerClip {
    active: Vec<(f32, bool)>,
    position: Vec<(f32, Vec3)>,
    rotation: Vec<(f32, Quat)>,
    scale: Vec<(f32, Vec3)>,
    spritesheet_index: Vec<(f32, SpritesheetIndex)>,
    uv_flip: Vec<(f32, bool)>,
    tint: Vec<(f32, LinearRgba)>,
}

#[derive(Debug)]
pub enum AssetLoaderError {
    Act(crate::Error),
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

impl From<crate::Error> for AssetLoaderError {
    fn from(value: crate::Error) -> Self {
        Self::Act(value)
    }
}

impl From<std::io::Error> for AssetLoaderError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<TryFromIntError> for AssetLoaderError {
    fn from(_value: TryFromIntError) -> Self {
        Self::UsizeConversion
    }
}
