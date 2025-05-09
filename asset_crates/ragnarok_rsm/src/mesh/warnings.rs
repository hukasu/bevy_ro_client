use std::fmt::Display;

#[derive(Debug)]
pub enum Warning {
    TextureOutOfBounds(usize, i32),
    CantBeAddressed(i32),
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextureOutOfBounds(length, index) => {
                write!(
                    f,
                    "Mesh uses texture {}, but there are only {} available.",
                    index, length,
                )
            }
            Self::CantBeAddressed(index) => {
                write!(
                    f,
                    "Texture {} can't be addressed on this architecture.",
                    index,
                )
            }
        }
    }
}

impl ragnarok_rebuild_common::warning::Warning for Warning {}
