use std::{io::Error, path::Path};

use bevy_asset::io::{
    AssetReader as BevyAsserReader, AssetReaderError, PathStream, Reader, VecReader,
};

pub struct AssetReader {
    grf: super::Grf,
}

impl AssetReader {
    pub fn new(path: &Path) -> Result<Self, super::Error> {
        Ok(Self {
            grf: super::Grf::new(path)?,
        })
    }
}

impl BevyAsserReader for AssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<Box<dyn Reader + 'a>, AssetReaderError> {
        match self.grf.read_file(path) {
            Ok(data) => {
                let reader: Box<dyn Reader> = Box::new(VecReader::new(data.to_vec()));
                Ok(reader)
            }
            Err(super::Error::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
            Err(err) => Err(AssetReaderError::Io(
                Error::other(format!(
                    "An error occurred while checking if path is directory. '{err}'"
                ))
                .into(),
            )),
        }
    }

    async fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<dyn Reader + 'a>, AssetReaderError> {
        Err(AssetReaderError::NotFound(path.to_path_buf()))
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        match self.grf.is_directory(path) {
            Ok(is_dir) => Ok(is_dir),
            Err(super::Error::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
            Err(err) => Err(AssetReaderError::Io(
                Error::other(format!(
                    "An error occurred while checking if path is directory. '{err}'"
                ))
                .into(),
            )),
        }
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        match self.grf.read_directory(path) {
            Ok(paths) => {
                let stream: Box<PathStream> = Box::new(futures::stream::iter(paths.to_vec()));
                Ok(stream)
            }
            Err(super::Error::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
            Err(err) => Err(AssetReaderError::Io(
                Error::other(format!(
                    "An error occurred while checking if path is directory. '{err}'"
                ))
                .into(),
            )),
        }
    }
}
