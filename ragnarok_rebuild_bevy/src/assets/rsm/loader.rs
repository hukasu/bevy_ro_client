use bevy::{
    animation::{AnimationClip, AnimationPlayer, EntityPath, VariableCurve},
    asset::{io::Reader, AssetLoader as BevyAssetLoader, AsyncReadExt, Handle, LoadContext},
    core::Name,
    ecs::world::{EntityWorldMut, World},
    hierarchy::{BuildWorldChildren, WorldChildBuilder},
    math::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4},
    pbr::{PbrBundle, StandardMaterial},
    prelude::SpatialBundle,
    render::{mesh::Mesh, primitives::Aabb, render_resource::PrimitiveTopology, texture::Image},
    scene::Scene,
    transform::components::Transform,
    utils::hashbrown::{hash_map::Entry, HashMap},
};
use ragnarok_rebuild_common::assets::rsm;

use crate::assets::paths;

pub struct AssetLoader;

impl AssetLoader {
    fn generate_model(rsm: &rsm::RSM, load_context: &mut LoadContext) -> Scene {
        bevy::log::trace!("Generating animated prop {:?}.", load_context.path());
        let mut world = World::new();

        let textures = Self::load_textures(&rsm.textures, load_context);

        let mut parent = world.spawn((Name::new("root"), SpatialBundle::INHERITED_IDENTITY));
        parent.with_children(|parent| {
            for root_mesh in rsm
                .meshes
                .iter()
                .filter(|mesh| rsm.root_meshes.contains(&mesh.name))
            {
                Self::build_mesh(
                    rsm,
                    root_mesh,
                    &textures,
                    rsm.shade_type,
                    parent,
                    load_context,
                );
            }
        });

        Self::build_animation(rsm, &mut parent, load_context);

        Scene { world }
    }

    fn load_textures(paths: &[Box<str>], load_context: &mut LoadContext) -> Vec<Handle<Image>> {
        bevy::log::trace!(
            "Loading textures for animated prop {:?}.",
            load_context.path()
        );
        paths
            .iter()
            .map(|texture_path| {
                load_context.load(format!("{}{texture_path}", paths::TEXTURE_FILES_FOLDER))
            })
            .collect::<Vec<_>>()
    }

    fn build_animation(
        rsm: &rsm::RSM,
        parent: &mut EntityWorldMut,
        load_context: &mut LoadContext,
    ) {
        bevy::log::trace!(
            "Generating animation for animated prop {:?}.",
            load_context.path()
        );

        let mut clip = AnimationClip::default();

        // RSM Scale
        clip.add_curve_to_path(
            EntityPath {
                parts: vec!["root".into()],
            },
            VariableCurve {
                keyframe_timestamps: rsm
                    .scale_key_frames
                    .iter()
                    .map(|frame| frame.frame as f32 / 1000.)
                    .collect(),
                keyframes: bevy::animation::Keyframes::Scale(
                    rsm.scale_key_frames
                        .iter()
                        .map(|frame| Vec3::from_array(frame.scale))
                        .collect(),
                ),
            },
        );

        let animation = load_context.add_labeled_asset("Animation0".to_owned(), clip.clone());

        parent.insert((AnimationPlayer::default(), super::Model { animation }));
    }

    fn build_mesh(
        rsm: &rsm::RSM,
        rsm_mesh: &rsm::mesh::Mesh,
        rsm_textures: &[Handle<Image>],
        shade_type: rsm::ShadeType,
        parent: &mut WorldChildBuilder,
        load_context: &mut LoadContext,
    ) {
        bevy::log::trace!(
            "Generating mesh '{}' for animated prop {:?}.",
            rsm_mesh.name,
            load_context.path()
        );

        let Some(mesh_bounds) = Self::mesh_bounds(rsm_mesh) else {
            bevy::log::warn!(
                "Mesh {} from model's {:?} had no vertexes.",
                rsm_mesh.name,
                load_context.path()
            );
            return;
        };

        let node_transform = if rsm_mesh.parent_name.is_empty() {
            Self::recentered_mesh_transform(rsm_mesh, &mesh_bounds)
        } else {
            Self::mesh_transform(rsm_mesh)
        };

        let mut node = parent.spawn((
            Name::new(rsm_mesh.name.to_string()),
            SpatialBundle::from_transform(node_transform),
        ));

        let mesh_textures = if rsm_mesh.textures.is_empty() {
            rsm_mesh
                .texture_indexes
                .iter()
                .map(|id| rsm_textures[*id as usize].clone())
                .collect()
        } else {
            Self::load_textures(&rsm_mesh.textures, load_context)
        };

        let mesh_vertexes = rsm_mesh
            .vertices
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect::<Vec<_>>();
        let mesh_vertex_colors = rsm_mesh
            .uvs
            .iter()
            .map(|uv| uv.color.map(|channel| channel as f32 / 255.))
            .map(Vec4::from_array)
            .collect::<Vec<_>>();
        let mesh_uvs = rsm_mesh
            .uvs
            .iter()
            .map(|uv| uv.uv)
            .map(Vec2::from_array)
            .collect::<Vec<_>>();

        node.with_children(|parent| {
            let transform = Self::mesh_transformation_matrix(rsm_mesh);
            parent
                .spawn((
                    Name::new("Primitives"),
                    SpatialBundle {
                        transform,
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    for (i, ((texture_id, two_sided), primitive_faces)) in
                        Self::split_mesh_into_primitives(rsm_mesh)
                            .iter()
                            .enumerate()
                    {
                        let mesh = load_context.add_labeled_asset(
                            format!("{}Primitive{}", rsm_mesh.name, i),
                            match shade_type {
                                rsm::ShadeType::Unlit | rsm::ShadeType::Flat => Self::flat_mesh(
                                    primitive_faces,
                                    &mesh_vertexes,
                                    &mesh_vertex_colors,
                                    &mesh_uvs,
                                ),
                                rsm::ShadeType::Smooth => Self::flat_mesh(
                                    primitive_faces,
                                    &mesh_vertexes,
                                    &mesh_vertex_colors,
                                    &mesh_uvs,
                                ),
                            },
                        );
                        let material = load_context.add_labeled_asset(
                            format!("{}Material{}", rsm_mesh.name, i),
                            Self::mesh_material(
                                mesh_textures[*texture_id as usize].clone(),
                                shade_type == rsm::ShadeType::Unlit,
                                two_sided == &1,
                            ),
                        );

                        parent.spawn((
                            Name::new(format!("Primitive{i}")),
                            PbrBundle {
                                mesh,
                                material,
                                ..Default::default()
                            },
                        ));
                    }
                });

            // Spawn children nodes
            for child_mesh in rsm
                .meshes
                .iter()
                .filter(|mesh| (*mesh.parent_name).eq(&*rsm_mesh.name))
            {
                Self::build_mesh(
                    rsm,
                    child_mesh,
                    &rsm_textures,
                    shade_type,
                    parent,
                    load_context,
                );
            }
        });
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn split_mesh_into_primitives(
        mesh: &rsm::mesh::Mesh,
    ) -> HashMap<(u16, u8), Vec<&rsm::mesh::Face>> {
        mesh.faces.iter().fold(HashMap::new(), |mut hm, face| {
            match hm.entry((face.texture_id, face.two_side)) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![face]);
                }
                Entry::Occupied(mut entry) => entry.get_mut().push(face),
            }

            hm
        })
    }

    #[must_use]
    fn mesh_transform(mesh: &rsm::mesh::Mesh) -> Transform {
        let translation = Vec3::from_array(mesh.position);
        let rotation = {
            let rotation_axis = Vec3::from_array(mesh.rotation_axis);
            if rotation_axis.length() <= 0. {
                Quat::default()
            } else {
                Quat::from_axis_angle(rotation_axis, mesh.rotation_angle)
            }
        };
        let scale = Vec3::from_array(mesh.scale);

        Transform {
            translation,
            rotation,
            scale,
        }
    }

    #[must_use]
    fn recentered_mesh_transform(mesh: &rsm::mesh::Mesh, mesh_bounds: &Aabb) -> Transform {
        let translation = Vec3::from_array(mesh.position)
            - Vec3::new(
                mesh_bounds.center.x,
                mesh_bounds.max().y,
                mesh_bounds.center.z,
            );
        let rotation = {
            let rotation_axis = Vec3::from_array(mesh.rotation_axis);
            if rotation_axis.length() <= 0. {
                Quat::default()
            } else {
                Quat::from_axis_angle(rotation_axis, mesh.rotation_angle)
            }
        };
        let scale = Vec3::from_array(mesh.scale);

        Transform {
            translation,
            rotation,
            scale,
        }
    }

    #[must_use]
    fn mesh_transformation_matrix(mesh: &rsm::mesh::Mesh) -> Transform {
        let offset = Vec3::from_array(mesh.offset);
        let trasn_matrix = Mat3::from_cols_array(&mesh.transformation_matrix);
        Transform::from_matrix(Mat4 {
            x_axis: trasn_matrix.x_axis.extend(0.),
            y_axis: trasn_matrix.y_axis.extend(0.),
            z_axis: trasn_matrix.z_axis.extend(0.),
            w_axis: offset.extend(1.),
        })
    }

    #[must_use]
    fn flat_mesh(
        primitive_faces: &[&rsm::mesh::Face],
        mesh_vertexes: &[Vec3],
        mesh_vertex_colors: &[Vec4],
        mesh_uvs: &[Vec2],
    ) -> Mesh {
        let mut vertexes = vec![];
        let mut uvs = vec![];
        let mut vertex_colors = vec![];
        let mut normals = vec![];

        for rsm::mesh::Face {
            vertices: face_vertex_ids,
            uv: face_uvs_ids,
            texture_id: _,
            two_side: _,
            smoothing_group: _,
        } in primitive_faces.iter()
        {
            let mut face_vertexes = face_vertex_ids
                .map(|id| mesh_vertexes[id as usize])
                .to_vec();
            let mut face_uvs = face_uvs_ids.map(|id| mesh_uvs[id as usize]).to_vec();
            let mut face_colors = face_uvs_ids
                .map(|id| mesh_vertex_colors[id as usize])
                .to_vec();

            normals.append(&mut vec![
                Self::face_normal(
                    face_vertexes[0],
                    face_vertexes[1],
                    face_vertexes[2]
                );
                3
            ]);
            vertexes.append(&mut face_vertexes);
            uvs.append(&mut face_uvs);
            vertex_colors.append(&mut face_colors);
        }

        Mesh::new(PrimitiveTopology::TriangleList)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertexes)
            // .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colors)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    }

    #[must_use]
    fn smooth_mesh(
        face_tris: &[(&[u16; 3], &[u16; 3], &[i32])],
        mesh_vertexes: &[Vec3],
        mesh_vertex_colors: &[Vec4],
        mesh_uvs: &[Vec2],
        shade_type: rsm::ShadeType,
        normal: Vec3,
    ) -> Mesh {
        todo!()
    }

    #[must_use]
    fn mesh_material(texture: Handle<Image>, unlit: bool, double_sided: bool) -> StandardMaterial {
        StandardMaterial {
            base_color_texture: Some(texture),
            double_sided,
            cull_mode: None,
            unlit,
            reflectance: 0.2,
            ..Default::default()
        }
    }

    #[must_use]
    fn smoothing_groups_normals(
        mesh: &rsm::mesh::Mesh,
        mesh_vertexes: &[Vec3],
    ) -> HashMap<i32, Vec3> {
        let mut normals = mesh.faces.iter().fold(HashMap::new(), |mut hm, face| {
            let face_normal = Self::face_normal(
                mesh_vertexes[face.vertices[0] as usize],
                mesh_vertexes[face.vertices[1] as usize],
                mesh_vertexes[face.vertices[2] as usize],
            );

            for smoothing_group in face.smoothing_group.iter() {
                match hm.entry(*smoothing_group) {
                    Entry::Vacant(entry) => {
                        entry.insert(face_normal);
                    }
                    Entry::Occupied(mut entry) => {
                        *entry.get_mut() += face_normal;
                    }
                }
            }
            hm
        });
        normals.iter_mut().for_each(|(_, normal)| {
            *normal = normal.normalize();
        });
        normals
    }

    #[must_use]
    fn face_normal(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
        let u = v2 - v1;
        let v = v3 - v1;
        u.cross(v).normalize()
    }

    #[must_use]
    fn mesh_bounds(mesh: &rsm::mesh::Mesh) -> Option<Aabb> {
        let transformation_matrix = Self::mesh_transformation_matrix(mesh);
        let transform = Self::mesh_transform(mesh);

        Aabb::enclosing(mesh.vertices.iter().map(move |vertex| {
            transform
                .transform_point(transformation_matrix.transform_point(Vec3::from_array(*vertex)))
        }))
    }
}

impl BevyAssetLoader for AssetLoader {
    type Asset = Scene;
    type Settings = ();
    type Error = rsm::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async {
            let mut data: Vec<u8> = vec![];
            reader.read_to_end(&mut data).await?;
            let rsm = rsm::RSM::from_reader(&mut data.as_slice())?;

            Ok(Self::generate_model(&rsm, load_context))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rsm", "rsm2"]
    }
}
