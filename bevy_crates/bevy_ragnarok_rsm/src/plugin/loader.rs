use std::path::{Path, PathBuf};

use bevy_animation::{
    AnimationClip, AnimationPlayer, AnimationTarget, AnimationTargetId,
    graph::{AnimationGraph, AnimationGraphHandle},
};
use bevy_asset::{AssetLoader as BevyAssetLoader, Handle, LoadContext, io::Reader};
use bevy_camera::visibility::Visibility;
use bevy_ecs::{
    bundle::Bundle,
    entity::Entity,
    hierarchy::{ChildOf, Children},
    name::Name,
    spawn::{SpawnIter, SpawnRelated, SpawnableList},
    world::World,
};
use bevy_image::Image;
use bevy_mesh::Mesh3d;
use bevy_platform::collections::{HashMap, HashSet};
use bevy_scene::Scene;
use bevy_transform::components::Transform;
use ragnarok_rsm::{Rsm, ShadeType, mesh::Textures};

use crate::{
    Model, ModelAnimation, RsmMaterials,
    assets::RsmModel,
    extensions::{RsmExt, RsmMeshExt, RsmPrimitiveExt},
    materials::RsmMaterial,
};

type TextureCache = HashMap<(Handle<Image>, bool), (Handle<RsmMaterial>, Handle<RsmMaterial>)>;

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
    type Asset = RsmModel;
    type Settings = ();
    type Error = ragnarok_rsm::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut data: Vec<u8> = vec![];
        reader.read_to_end(&mut data).await?;

        let rsm = ragnarok_rsm::Rsm::from_reader(&mut data.as_slice())?;

        let scene = {
            let scene = SceneBuilder::build(&rsm, load_context, self);
            load_context.add_labeled_asset("Scene".to_owned(), scene)
        };

        Ok(RsmModel { model: scene })
    }

    fn extensions(&self) -> &[&str] {
        &["rsm", "rsm2"]
    }
}

pub struct SceneBuilder;

impl SceneBuilder {
    pub fn build(rsm: &Rsm, load_context: &mut LoadContext<'_>, loader: &AssetLoader) -> Scene {
        log::trace!("Generating animated prop {:?}.", load_context.path());
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

        let mut texture_cache = TextureCache::new();

        world.entity_mut(root).insert((
            root_name,
            Transform::default(),
            Visibility::default(),
            animation,
            Children::spawn(MeshList::new(
                rsm,
                root_meshes.as_slice(),
                &mut meshes_and_indexes,
                root,
                &mut texture_cache,
                load_context,
                loader,
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

            if let Some(animation_curve) = rsm.position_animation_curve() {
                let root_mesh = rsm.root_meshes[0].to_string();
                let animation_target_id = AnimationTargetId::from_name(&Name::new(root_mesh));
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

#[derive(Clone)]
struct MeshList {
    meshes: Vec<MeshListItem>,
}

#[derive(Clone)]
struct MeshListItem {
    name: Name,
    transform: Transform,
    primitives: PrimitiveList,
    animation_player: Entity,
    children: MeshList,
}

impl MeshList {
    pub fn new(
        rsm: &Rsm,
        to_build: &[&str],
        remaining_meshes: &mut Vec<(usize, &str)>,
        animation_player: Entity,
        texture_cache: &mut TextureCache,
        load_context: &mut LoadContext,
        loader: &AssetLoader,
    ) -> Self {
        let mut mesh_list = Vec::new();

        for mesh_to_build in to_build {
            let Some(pos) = remaining_meshes
                .iter()
                .position(|(_, mesh)| *mesh == *mesh_to_build)
            else {
                continue;
            };
            let (mesh_index, mesh_name) = remaining_meshes.remove(pos);
            assert_eq!(
                *mesh_to_build, mesh_name,
                "Position had mesh name different from mesh_to_build."
            );
            let current_mesh = &rsm.meshes[mesh_index];

            let node_transform = if current_mesh.parent_name.is_empty() {
                let Some(mesh_bounds) = current_mesh.bounds() else {
                    log::warn!(
                        "Mesh {} from model's {:?} had no vertices.",
                        current_mesh.name,
                        load_context.path()
                    );
                    continue;
                };
                current_mesh.recentered_transform(&mesh_bounds)
            } else {
                current_mesh.transform()
            };

            let remaining_meshes_index = remaining_meshes
                .iter()
                .map(|(i, _)| i)
                .copied()
                .collect::<HashSet<_>>();
            let children = rsm
                .meshes
                .iter()
                .enumerate()
                .filter(|(i, _)| remaining_meshes_index.contains(i))
                .filter(|(_, mesh)| mesh.parent_name.as_ref() == *mesh_to_build)
                .map(|(_, mesh)| mesh.name.as_ref())
                .collect::<Vec<_>>();

            mesh_list.push(MeshListItem {
                name: Name::new(current_mesh.name.to_string()),
                transform: node_transform,
                primitives: PrimitiveList::new(
                    rsm,
                    mesh_index,
                    texture_cache,
                    load_context,
                    loader,
                ),
                animation_player,
                children: MeshList::new(
                    rsm,
                    children.as_slice(),
                    remaining_meshes,
                    animation_player,
                    texture_cache,
                    load_context,
                    loader,
                ),
            });
        }

        Self { meshes: mesh_list }
    }
}

impl SpawnableList<ChildOf> for MeshList {
    fn spawn(this: bevy_ecs::ptr::MovingPtr<'_, Self>, world: &mut World, entity: Entity) {
        for item in &this.meshes {
            let animation_target = AnimationTarget {
                id: AnimationTargetId::from_name(&item.name),
                player: item.animation_player,
            };
            world.spawn((
                ChildOf(entity),
                item.name.clone(),
                item.transform,
                Visibility::default(),
                animation_target,
                Children::spawn((item.primitives.clone(), item.children.clone())),
            ));
        }
    }

    fn size_hint(&self) -> usize {
        self.meshes.len()
    }
}

#[derive(Clone)]
struct PrimitiveList {
    transform: Transform,
    primitives: Vec<PrimitiveListItem>,
}

#[derive(Clone)]
struct PrimitiveListItem {
    name: Name,
    mesh: Handle<bevy_mesh::Mesh>,
    material: Handle<RsmMaterial>,
    inverted_material: Handle<RsmMaterial>,
}

impl PrimitiveList {
    pub fn new(
        rsm: &Rsm,
        mesh_index: usize,
        texture_cache: &mut TextureCache,
        load_context: &mut LoadContext,
        loader: &AssetLoader,
    ) -> Self {
        let mut primitive_list = Vec::new();

        let rsm_mesh = &rsm.meshes[mesh_index];

        let textures = match &rsm_mesh.textures {
            Textures::Paths(paths) => paths,
            Textures::Indexes(_) => &rsm.textures,
        };

        let mesh_attributes = if rsm.shade_type == ShadeType::Smooth {
            // TODO smooth normals
            rsm_mesh.flat_mesh()
        } else {
            rsm_mesh.flat_mesh()
        };

        for (id, primitive) in mesh_attributes.primitives.into_iter().enumerate() {
            let texture_id = primitive.texture_id;
            let double_sided = primitive.double_sided;
            let mesh = primitive.into_mesh();

            let texture_count = texture_cache.len();
            let Ok(texture_id) = usize::try_from(texture_id) else {
                log::warn!("Texture can't be indexed on current architecture.");
                continue;
            };
            let texture = load_context.load(
                loader
                    .texture_path_prefix()
                    .join(textures[texture_id].as_ref()),
            );

            let transparency = textures[texture_id].ends_with("tga");

            let material = texture_cache
                .entry((texture.clone(), double_sided))
                .or_insert((
                    load_context.add_labeled_asset(
                        format!("Material{}", texture_count),
                        RsmMaterial {
                            texture: texture.clone(),
                            double_sided,
                            inverse_scale: false,
                            transparency,
                        },
                    ),
                    load_context.add_labeled_asset(
                        format!("Material{}/Inverted", texture_count),
                        RsmMaterial {
                            texture: texture.clone(),
                            double_sided,
                            inverse_scale: true,
                            transparency,
                        },
                    ),
                ));

            primitive_list.push(PrimitiveListItem {
                name: Name::new(format!("Primitive{}", id)),
                mesh: load_context
                    .add_labeled_asset(format!("Mesh{}/Primitive{}/Mesh", mesh_index, id), mesh),
                material: material.0.clone(),
                inverted_material: material.1.clone(),
            });
        }

        Self {
            transform: Transform::from_matrix(rsm_mesh.transformation_matrix()),
            primitives: primitive_list,
        }
    }
}

impl SpawnableList<ChildOf> for PrimitiveList {
    fn spawn(this: bevy_ecs::ptr::MovingPtr<'_, Self>, world: &mut World, entity: Entity) {
        world.spawn((
            ChildOf(entity),
            Name::new("Primitives"),
            this.transform,
            Visibility::default(),
            Children::spawn(SpawnIter(this.primitives.clone().into_iter().map(|item| {
                (
                    item.name,
                    Mesh3d(item.mesh),
                    RsmMaterials {
                        base: item.material,
                        inverted: item.inverted_material,
                    },
                )
            }))),
        ));
    }

    fn size_hint(&self) -> usize {
        self.primitives.len()
    }
}
