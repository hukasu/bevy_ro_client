use std::{
    io::{Error, ErrorKind},
    path::Path,
};

use bevy::{
    asset::io::{AssetReader as BevyAsserReader, AssetReaderError, PathStream, Reader, VecReader},
    utils::BoxedFuture,
};

pub struct AssetReader {
    grf: super::GRF,
}

impl AssetReader {
    pub fn new(path: &Path) -> Result<Self, super::Error> {
        Ok(Self {
            grf: super::GRF::new(path)?,
        })
    }
}

impl BevyAsserReader for AssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async {
            match self.grf.read_file(path) {
                Ok(data) => {
                    let reader: Box<Reader<'a>> = Box::new(VecReader::new(data.to_vec()));
                    Ok(reader)
                }
                Err(super::Error::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
                Err(err) => Err(AssetReaderError::Io(Error::new(
                    ErrorKind::Other,
                    format!("An error occurred while checking if path is directory. '{err}'"),
                ))),
            }
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<Reader<'a>>, AssetReaderError>> {
        Box::pin(async { Err(AssetReaderError::NotFound(path.to_path_buf())) })
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        Box::pin(async move {
            match self.grf.is_directory(path) {
                Ok(is_dir) => Ok(is_dir),
                Err(super::Error::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
                Err(err) => Err(AssetReaderError::Io(Error::new(
                    ErrorKind::Other,
                    format!("An error occurred while checking if path is directory. '{err}'"),
                ))),
            }
        })
    }

    fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<PathStream>, AssetReaderError>> {
        Box::pin(async {
            match self.grf.read_directory(path) {
                Ok(paths) => {
                    let stream: Box<PathStream> = Box::new(futures::stream::iter(paths.to_vec()));
                    Ok(stream)
                }
                Err(super::Error::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
                Err(err) => Err(AssetReaderError::Io(Error::new(
                    ErrorKind::Other,
                    format!("An error occurred while checking if path is directory. '{err}'"),
                ))),
            }
        })
    }
}
