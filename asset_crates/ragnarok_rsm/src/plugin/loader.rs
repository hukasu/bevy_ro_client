use std::path::{Path, PathBuf};

use bevy_animation::{
    animated_field,
    graph::{AnimationGraph, AnimationGraphHandle},
    prelude::{AnimatableCurve, AnimatedField, AnimationCurve, AnimationTransitions},
    AnimationClip, AnimationPlayer, AnimationTarget, AnimationTargetId,
};
use bevy_asset::{io::Reader, AssetLoader as BevyAssetLoader, Handle, LoadContext};
use bevy_color::Color;
use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    hierarchy::{ChildOf, Children},
    name::Name,
    spawn::{SpawnRelated, SpawnableList},
    world::World,
};
use bevy_image::Image;
use bevy_math::prelude::Cuboid;
use bevy_mesh::{MeshBuilder, Meshable};
use bevy_pbr::{MeshMaterial3d, StandardMaterial};
use bevy_platform::collections::HashSet;
use bevy_render::{mesh::Mesh3d, view::Visibility};
use bevy_scene::Scene;
use bevy_transform::components::Transform;

use crate::Rsm;
use crate::{
    components::{Model, ModelAnimation},
    mesh::Mesh,
};

pub struct AssetLoader {
    texture_path_prefix: PathBuf,
}

impl AssetLoader {
    pub fn new(texture_path_prefix: PathBuf) -> Self {
        Self {
            texture_path_prefix,
        }
    }

    pub fn texture_path_prefix(&self) -> &Path {
        &self.texture_path_prefix
    }
}

impl BevyAssetLoader for AssetLoader {
    type Asset = Rsm;
    type Settings = ();
    type Error = crate::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let rsm = crate::Rsm::from_reader(&mut data.as_slice())?;

        let scene = SceneBuilder::build(&rsm, load_context, self);
        load_context.add_labeled_asset("Scene".to_owned(), scene);

        Ok(rsm)
    }

    fn extensions(&self) -> &[&str] {
        &["rsm", "rsm2"]
    }
}

pub struct SceneBuilder;

impl SceneBuilder {
    pub fn build(rsm: &Rsm, load_context: &mut LoadContext<'_>, loader: &AssetLoader) -> Scene {
        bevy_log::trace!("Generating animated prop {:?}.", load_context.path());
        let mut world = World::new();

        let root_name = Name::new("root");
        let root = world.spawn_empty().id();

        let mut meshes_and_indexes = rsm
            .meshes
            .iter()
            .enumerate()
            .map(|(i, mesh)| (i, mesh.name.as_ref()))
            .collect::<Vec<_>>();
        let root_meshes = rsm
            .root_meshes
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();

        let animation = Self::build_animation(rsm, root, &root_name, load_context);

        world.entity_mut(root).insert((
            root_name,
            Transform::default(),
            Visibility::default(),
            animation,
            Children::spawn(MeshList::new(
                &rsm.meshes,
                root_meshes.as_slice(),
                &mut meshes_and_indexes,
                root,
                load_context,
            )),
        ));

        Scene { world }
    }

    fn build_animation(
        rsm: &Rsm,
        root: Entity,
        root_name: &Name,
        load_context: &mut LoadContext<'_>,
    ) -> impl Bundle {
        let animation_clip = Self::build_animation_clip(rsm, load_context);

        let (animation_graph, model_animation) = if let Some(animation_clip) = animation_clip {
            let (animation_graph, node_index) = AnimationGraph::from_clip(animation_clip.clone());
            (
                animation_graph,
                Some(ModelAnimation {
                    animation: animation_clip,
                    animation_node_index: node_index,
                }),
            )
        } else {
            (AnimationGraph::new(), None)
        };
        let animation_target = AnimationTarget {
            id: AnimationTargetId::from_name(root_name),
            player: root,
        };

        (
            AnimationPlayer::default(),
            AnimationTransitions::new(),
            AnimationGraphHandle(
                load_context.add_labeled_asset("AnimationGraph".to_string(), animation_graph),
            ),
            animation_target,
            Model {
                animation: model_animation,
            },
        )
    }

    fn build_animation_clip(
        rsm: &Rsm,
        load_context: &mut LoadContext<'_>,
    ) -> Option<Handle<AnimationClip>> {
        {
            let mut animation_clip = AnimationClip::default();

            let mut populated = false;

            if let Some(animation_curve) = rsm.scale_animation_curve() {
                let animation_target_id = AnimationTargetId::from_name(&Name::new("root"));
                animation_clip.add_curve_to_target(animation_target_id, animation_curve);
                populated = true;
            }

            let animation_duration = rsm.animation_duration;

            for mesh in rsm.meshes.iter() {
                let id = AnimationTargetId::from_name(&Name::new(mesh.name.to_string()));
                if let Some(position) = mesh.position_animation_curve(animation_duration) {
                    animation_clip.add_curve_to_target(id, position);
                    populated = true;
                }
                if let Some(rotation) = mesh.rotation_animation_curve(animation_duration) {
                    animation_clip.add_curve_to_target(id, rotation);
                    populated = true;
                }
                if let Some(scale) = mesh.scale_animation_curve(animation_duration) {
                    animation_clip.add_curve_to_target(id, scale);
                    populated = true;
                }
            }

            if populated {
                animation_clip.set_duration(animation_duration.duration());
                Some(load_context.add_labeled_asset("Animation".to_string(), animation_clip))
            } else {
                None
            }
        }
    }
}

struct MeshList {
    meshes: Vec<MeshListItem>,
}

struct MeshListItem {
    name: Name,
    transform: Transform,
    mesh: Handle<bevy_mesh::Mesh>,
    material: Handle<StandardMaterial>,
    animation_player: Entity,
    children: MeshList,
}

impl MeshList {
    pub fn new(
        meshes: &[Mesh],
        to_build: &[&str],
        remaining_meshes: &mut Vec<(usize, &str)>,
        animation_player: Entity,
        load_context: &mut LoadContext,
    ) -> Self {
        let mut mesh_list = Vec::new();

        for mesh_to_build in to_build {
            let Some(pos) = remaining_meshes
                .iter()
                .position(|(_, mesh)| *mesh == *mesh_to_build)
            else {
                continue;
            };
            let (i, mesh_name) = remaining_meshes.remove(pos);
            assert_eq!(
                *mesh_to_build, mesh_name,
                "Position had mesh name different from mesh_to_build."
            );
            let current_mesh = &meshes[i];

            let Some(mesh_bounds) = current_mesh.bounds() else {
                bevy_log::warn!(
                    "Mesh {} from model's {:?} had no vertexes.",
                    current_mesh.name,
                    load_context.path()
                );
                continue;
            };

            let node_transform = if current_mesh.parent_name.is_empty() {
                current_mesh.recentered_transform(&mesh_bounds)
            } else {
                current_mesh.transform()
            };

            let remaining_meshes_index = remaining_meshes
                .iter()
                .map(|(i, _)| i)
                .copied()
                .collect::<HashSet<_>>();
            let children = meshes
                .iter()
                .enumerate()
                .filter(|(i, _)| remaining_meshes_index.contains(i))
                .filter(|(_, mesh)| mesh.parent_name.as_ref() == *mesh_to_build)
                .map(|(_, mesh)| mesh.name.as_ref())
                .collect::<Vec<_>>();

            mesh_list.push(MeshListItem {
                name: Name::new(current_mesh.name.to_string()),
                transform: node_transform,
                mesh: load_context.add_labeled_asset(
                    format!("Mesh{}", i),
                    Cuboid::new(10., 10., 10.).mesh().build(),
                ),
                material: load_context
                    .add_labeled_asset(format!("Material{}", i), Color::WHITE.into()),
                animation_player,
                children: MeshList::new(
                    meshes,
                    children.as_slice(),
                    remaining_meshes,
                    animation_player,
                    load_context,
                ),
            });
        }

        Self { meshes: mesh_list }
    }
}

impl SpawnableList<ChildOf> for MeshList {
    fn spawn(self, world: &mut World, entity: Entity) {
        for item in self.meshes {
            let animation_target = AnimationTarget {
                id: AnimationTargetId::from_name(&item.name),
                player: item.animation_player,
            };
            world.spawn((
                ChildOf(entity),
                item.name,
                item.transform,
                Mesh3d(item.mesh),
                MeshMaterial3d(item.material),
                animation_target,
                Children::spawn(item.children),
            ));
        }
    }

    fn size_hint(&self) -> usize {
        self.meshes.len()
    }
}

#[derive(Debug, Clone)]
enum LoadedTextureFormat {
    Bmp,
    Tga,
}

#[derive(Debug, Clone)]
struct LoadedTexture {
    texture: Handle<Image>,
    format: LoadedTextureFormat,
}
