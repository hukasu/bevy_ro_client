use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            Self::Io(io) => format!("An IO error occurred while reading a Palette. '{io}'"),
        };
        write!(f, "{msg}")
    }
}

impl std::error::Error for Error {}
