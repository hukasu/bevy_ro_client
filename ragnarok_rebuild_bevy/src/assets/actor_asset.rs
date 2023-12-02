use bevy::{
    asset::{Asset, AssetLoader},
    reflect::TypePath,
};
use futures::AsyncReadExt;

use ragnarok_rebuild_common::act::{Actor, ActorError};

#[derive(Debug, Asset, TypePath)]
pub struct ActorAsset {
    actor: Actor,
}

pub struct ActorAssetLoader;

impl ActorAssetLoader {
    async fn load_actor<'a>(
        reader: &mut bevy::asset::io::Reader<'a>,
    ) -> Result<<Self as AssetLoader>::Asset, <Self as AssetLoader>::Error> {
        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).await?;
        Actor::from_bytes(&buffer).map(|actor| ActorAsset { actor })
    }
}

impl AssetLoader for ActorAssetLoader {
    type Asset = ActorAsset;
    type Settings = ();
    type Error = ActorError;

    fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(Self::load_actor(reader))
    }

    fn extensions(&self) -> &[&str] {
        &["act"]
    }
}
