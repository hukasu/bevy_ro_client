//! Allow for the use of Ragnarok Online's Grf files as
//! Bevy's [`AssetReader`].

use std::{io::Error, path::Path};

use bevy_asset::io::{
    AssetReader as BevyAssetReader, AssetReaderError, PathStream, Reader, VecReader,
};
use ragnarok_grf::{Error as GrfError, Grf};

pub struct AssetReader {
    grf: Grf,
}

impl AssetReader {
    pub fn new(path: &Path) -> Result<Self, GrfError> {
        Ok(Self {
            grf: Grf::new(path)?,
        })
    }
}

impl BevyAssetReader for AssetReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<Box<dyn Reader + 'a>, AssetReaderError> {
        log::trace!("Starting reading {}.", path.display());
        match self.grf.read_file(path) {
            Ok(data) => {
                let reader: Box<dyn Reader> = Box::new(VecReader::new(data.to_vec()));
                Ok(reader)
            }
            Err(GrfError::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
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
            Err(GrfError::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
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
            Err(GrfError::FileNotFound) => Err(AssetReaderError::NotFound(path.to_owned())),
            Err(err) => Err(AssetReaderError::Io(
                Error::other(format!(
                    "An error occurred while checking if path is directory. '{err}'"
                ))
                .into(),
            )),
        }
    }
}
