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
