use std::{collections::HashSet, fmt::Display, ops::Deref};

use ragnarok_rebuild_common::warning::ReportWarning;

use crate::{Rsm, mesh::Textures};

impl ReportWarning for Rsm {
    fn report(&self) -> impl Display {
        RsmWarning(self)
    }
}

struct RsmWarning<'a>(&'a Rsm);

impl Deref for RsmWarning<'_> {
    type Target = Rsm;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Display for RsmWarning<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.report_alpha(f)?;
        self.report_root_meshes(f)?;
        self.report_meshes(f)?;
        self.report_volume_boxes(f)?;

        Ok(())
    }
}

impl RsmWarning<'_> {
    fn report_alpha(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.alpha != 0xff {
            writeln!(f, "Alpha is different from 255, was {}.", self.alpha)?;
        }
        Ok(())
    }

    fn report_root_meshes(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.root_meshes.is_empty() {
            writeln!(f, "Has no root meshes.")?;
        }
        if self.root_meshes.len() > 1 {
            writeln!(f, "Has {} root meshes.", self.root_meshes.len())?;
        }
        if self.root_meshes.iter().any(|name| name.is_empty()) {
            writeln!(f, "Has root mesh with blank name.")?;
        }

        let mut names = HashSet::new();
        let mut reported_names = HashSet::new();
        for name in self.root_meshes.iter() {
            if !names.insert(name.as_ref()) && reported_names.insert(name.as_ref()) {
                writeln!(f, "Name {} appears multiple times on root meshes.", name)?;
            }
        }

        Ok(())
    }

    fn report_meshes(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut names = HashSet::new();
        let mut reported_names = HashSet::new();
        for mesh in self.meshes.iter() {
            if !names.insert(mesh.name.as_ref()) && reported_names.insert(mesh.name.as_ref()) {
                writeln!(f, "Name {:?} appears multiple times.", mesh.name)?;
            }

            if mesh.name.is_empty() {
                writeln!(f, "Mesh has blank name.")?;
            }

            if mesh.name == mesh.parent_name {
                writeln!(f, "Mesh {:?} has self reference.", mesh.name)?;
            }

            match &mesh.textures {
                Textures::Indexes(indexes) => {
                    for texture_index in indexes {
                        if *texture_index < 0 {
                            writeln!(f, "Uses negative texture index {}.", texture_index)?;
                        } else if usize::try_from(*texture_index).is_err() {
                            writeln!(
                                f,
                                "Texture index {} is not addressable on current architecture.",
                                texture_index
                            )?;
                        }
                    }
                }
                Textures::Paths(_) => (),
            }
        }

        Ok(())
    }

    fn report_volume_boxes(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(volume_boxes) = &self.volume_boxes {
            if !volume_boxes.is_empty() {
                writeln!(f, "Volume boxes is not empty.")?;
            }
        } else {
            writeln!(f, "Did not have volume box section.")?;
        }
        Ok(())
    }
}
