use bevy_asset::Asset;
use bevy_reflect::TypePath;

/// A [`Gat`] asset holding the source [`Gat`](ragnarok_gat::Gat).
#[expect(dead_code)]
#[derive(Debug, Asset, TypePath)]
pub struct Gat(pub ragnarok_gat::Gat);
