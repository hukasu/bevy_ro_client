use std::{fs::File, io::BufReader, sync::MutexGuard};

#[derive(Debug)]
pub enum Error {
    FileNotFound,
    WrongSignature,
    UnsupportedVersion,
    Io(std::io::Error),
    Zip(flate2::DecompressError),
    MutexPoisoned,
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<flate2::DecompressError> for Error {
    fn from(value: flate2::DecompressError) -> Self {
        Self::Zip(value)
    }
}

impl From<std::sync::PoisonError<MutexGuard<'_, BufReader<File>>>> for Error {
    fn from(_value: std::sync::PoisonError<MutexGuard<'_, BufReader<File>>>) -> Self {
        Self::MutexPoisoned
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Error::FileNotFound => "File not found within GRF.".to_owned(),
            Error::WrongSignature => "File had wrong signature.".to_owned(),
            Error::UnsupportedVersion => "GRF in on a unsupported version.".to_owned(),
            Error::Io(io) => format!("An IO error occured. '{io}'"),
            Error::Zip(zip) => format!("An error occured while deflating GRF. '{zip}'"),
            Error::MutexPoisoned => "The Mutex that hold the file is poisoned.".to_string(),
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}
