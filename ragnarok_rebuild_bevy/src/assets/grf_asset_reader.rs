use std::path::Path;

use bevy::{
    asset::io::{AssetReader, AssetReaderError, PathStream},
    utils::BoxedFuture,
};
use ragnarok_rebuild_common::grf::{GRFError, GRF};

pub struct GrfAssetReader {
    grf: ragnarok_rebuild_common::grf::GRF,
}

impl GrfAssetReader {
    pub fn new(path: &Path) -> Result<Self, GRFError> {
        Ok(Self {
            grf: GRF::new(path)?,
        })
    }
}

impl AssetReader for GrfAssetReader {
    fn read<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<bevy::asset::io::Reader<'a>>, AssetReaderError>> {
        Box::pin(async {
            match self.grf.read_file(path) {
                Ok(data) => {
                    let reader: Box<bevy::asset::io::Reader<'a>> =
                        Box::new(bevy::asset::io::VecReader::new(data.to_vec()));
                    Ok(reader)
                }
                Err(ragnarok_rebuild_common::grf::GRFError::FileNotFound) => {
                    Err(AssetReaderError::NotFound(path.to_owned()))
                }
                Err(err) => Err(AssetReaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("An error occurred while checking if path is directory. '{err}'"),
                ))),
            }
        })
    }

    fn read_meta<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<Box<bevy::asset::io::Reader<'a>>, AssetReaderError>> {
        Box::pin(async { Err(AssetReaderError::NotFound(path.to_path_buf())) })
    }

    fn is_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, Result<bool, AssetReaderError>> {
        Box::pin(async move {
            match self.grf.is_directory(path) {
                Ok(is_dir) => Ok(is_dir),
                Err(ragnarok_rebuild_common::grf::GRFError::FileNotFound) => {
                    Err(AssetReaderError::NotFound(path.to_owned()))
                }
                Err(err) => Err(AssetReaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
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
                Err(ragnarok_rebuild_common::grf::GRFError::FileNotFound) => {
                    Err(AssetReaderError::NotFound(path.to_owned()))
                }
                Err(err) => Err(AssetReaderError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("An error occurred while checking if path is directory. '{err}'"),
                ))),
            }
        })
    }
}
