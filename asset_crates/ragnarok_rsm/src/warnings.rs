use std::fmt::Display;

#[derive(Debug)]
pub enum Warning {
    EmptyRootMeshes,
    DuplicateRootMeshName(Box<str>),
    NonEmptyVolumeBox,
    MissingVolumeBoxSection,
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyRootMeshes => write!(f, "Root meshes is empty."),
            Self::DuplicateRootMeshName(name) => {
                write!(f, "Root mesh name \"{}\" appear multiple times.", name)
            }
            Self::NonEmptyVolumeBox => write!(f, "Has volume box."),
            Self::MissingVolumeBoxSection => write!(f, "Did not have a section for volume boxes."),
        }
    }
}

impl ragnarok_rebuild_common::warning::Warning for Warning {}
