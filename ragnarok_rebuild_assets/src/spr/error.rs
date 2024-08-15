use std::fmt::Display;

#[derive(Debug)]
pub enum SpriteError {
    RLE,
    Io(std::io::Error),
}

impl From<std::io::Error> for SpriteError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for SpriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::RLE => {
                "Size of image after RLE decompression does not match image dimensions".to_owned()
            }
            Self::Io(io) => format!("An IO error occurred while reading a Sprite. '{io}'"),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for SpriteError {}
