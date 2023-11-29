use std::{error::Error, fs::File, io::BufReader, sync::MutexGuard};

#[derive(Debug)]
pub enum GRFError<'a> {
    FileNotFound,
    WrongSignature,
    UnsupportedVersion,
    Io(std::io::Error),
    Zip(flate2::DecompressError),
    MutexPoisoned(std::sync::PoisonError<MutexGuard<'a, BufReader<File>>>),
}

impl<'a> From<std::io::Error> for GRFError<'a> {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl<'a> From<flate2::DecompressError> for GRFError<'a> {
    fn from(value: flate2::DecompressError) -> Self {
        Self::Zip(value)
    }
}

impl<'a> From<std::sync::PoisonError<MutexGuard<'a, BufReader<File>>>> for GRFError<'a> {
    fn from(value: std::sync::PoisonError<MutexGuard<'a, BufReader<File>>>) -> Self {
        Self::MutexPoisoned(value)
    }
}

impl<'a> std::fmt::Display for GRFError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            GRFError::FileNotFound => "File not found within GRF.".to_owned(),
            GRFError::WrongSignature => "File had wrong signature.".to_owned(),
            GRFError::UnsupportedVersion => "GRF in on a unsupported version.".to_owned(),
            GRFError::Io(io) => format!("An IO error occured. '{io}'"),
            GRFError::Zip(zip) => format!("An error occured while deflating GRF. '{zip}'"),
            GRFError::MutexPoisoned(poison) => {
                format!("The Mutex that hold the file is poisoned. '{poison}'")
            }
        };
        write!(f, "{message}")
    }
}

impl<'a> Error for GRFError<'a> {}

impl<'a> From<GRFError<'a>> for bevy::asset::io::AssetReaderError {
    fn from(value: GRFError) -> Self {
        match value {
            GRFError::FileNotFound => bevy::asset::io::AssetReaderError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find GRF file.",
            )),
            GRFError::WrongSignature => bevy::asset::io::AssetReaderError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Grf has wrong signature.",
            )),
            GRFError::UnsupportedVersion => {
                bevy::asset::io::AssetReaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Grf has unsupported version.",
                ))
            }
            GRFError::Io(io) => bevy::asset::io::AssetReaderError::Io(io),
            GRFError::Zip(_zip) => bevy::asset::io::AssetReaderError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Failed to decompress data from Grf.",
            )),
            GRFError::MutexPoisoned(_poison) => {
                bevy::asset::io::AssetReaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Mutex guarding BufReader is poisoned.",
                ))
            }
        }
    }
}
