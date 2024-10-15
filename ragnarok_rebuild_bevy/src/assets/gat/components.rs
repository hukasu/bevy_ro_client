use bevy::{
    math::Vec3A,
    prelude::{Component, ReflectComponent},
    reflect::Reflect,
    render::primitives::Aabb,
};
use ragnarok_rebuild_assets::gat;

#[derive(Debug, Component, Reflect)]
#[reflect(Component)]
pub struct Tiles {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
}

impl Tiles {
    pub fn iter_tiles(&self, aabb: Aabb) -> impl Iterator<Item = TileRef> + '_ {
        let min = (aabb.min() + Vec3A::new((self.width / 2) as f32, 0., (self.height / 2) as f32))
            .as_uvec3();
        let max = (aabb.max() + Vec3A::new((self.width / 2) as f32, 0., (self.height / 2) as f32))
            .as_uvec3();

        ((min.z as usize)..=(max.z as usize).min(self.height as usize)).flat_map(move |z| {
            ((min.x as usize)..=(max.x as usize).min(self.width as usize)).map(move |x| TileRef {
                x: x as u16,
                z: z as u16,
                tile: self.tiles[x + z * self.width as usize],
            })
        })
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
pub struct TileRef {
    pub x: u16,
    pub z: u16,
    pub tile: Tile,
}

// TODO change to gat::Tile when Bevy implements foreing type reflection
#[derive(Debug, Clone, Copy, Reflect)]
pub struct Tile {
    pub bottom_left: f32,
    pub bottom_right: f32,
    pub top_left: f32,
    pub top_right: f32,
    pub tile_type: u8,
    pub is_water_tile: bool,
}

impl From<&gat::Tile> for Tile {
    fn from(value: &gat::Tile) -> Self {
        Self {
            bottom_left: value.bottom_left_altitude(),
            bottom_right: value.bottom_right_altitude(),
            top_left: value.top_left_altitude(),
            top_right: value.top_right_altitude(),
            tile_type: value.tile_type(),
            is_water_tile: value.is_water_tile(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{math::Vec3, render::primitives::Aabb};

    use super::{Tile, Tiles};

    #[test]
    fn test_iterate_tiles() {
        let quad_tree = Tiles {
            width: 10,
            height: 10,
            tiles: (0..100)
                .map(|i| Tile {
                    bottom_left: 0.,
                    bottom_right: 0.,
                    top_left: 0.,
                    top_right: 0.,
                    tile_type: i,
                    is_water_tile: false,
                })
                .collect(),
        };
        // Iterating from the root should return all in order
        assert!((0..100)
            .zip(quad_tree.iter_tiles(Aabb::from_min_max(
                Vec3::new(-5., 0., -5.),
                Vec3::new(5., 0., 5.)
            )))
            .all(|(i, tile_ref)| tile_ref.tile.tile_type == i));
        // Iterating over lower left tiles
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(-5., 0., -5.),
                Vec3::new(0., 0., 0.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([
                0, 1, 2, 3, 4, 10, 11, 12, 13, 14, 20, 21, 22, 23, 24, 30, 31, 32, 33, 34, 40, 41,
                42, 43, 44,
            ])
            .all(|(tile, i)| tile == i));
        // Iterating over lower left tiles, but a bit shifted
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(-7., 0., -7.),
                Vec3::new(-2., 0., -2.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([0, 1, 2, 10, 11, 12, 20, 21, 22,])
            .all(|(tile, i)| tile == i));
        // Iterating over lower right tiles
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(0., 0., -5.),
                Vec3::new(5., 0., 0.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([
                5, 6, 7, 8, 9, 15, 16, 17, 18, 19, 25, 26, 27, 28, 29, 35, 36, 37, 38, 39, 45, 46,
                47, 48, 49
            ])
            .all(|(tile, i)| tile == i));
        // Iterating over lower right tiles, but a bit shifted
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(2., 0., -7.),
                Vec3::new(7., 0., -2.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([7, 8, 9, 17, 18, 19, 27, 28, 29,])
            .all(|(tile, i)| tile == i));
        // Iterating over top left tiles
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(-5., 0., 0.),
                Vec3::new(0., 0., 5.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([
                50, 51, 52, 53, 54, 60, 61, 62, 63, 64, 70, 71, 72, 73, 74, 80, 81, 82, 83, 84, 90,
                91, 92, 93, 94
            ])
            .all(|(tile, i)| tile == i));
        // Iterating over top left tiles, but a bit shifted
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(-7., 0., 2.),
                Vec3::new(-2., 0., 7.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([70, 71, 72, 80, 81, 82, 90, 91, 92,])
            .all(|(tile, i)| tile == i));
        // Iterating over top right tiles
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(0., 0., 0.),
                Vec3::new(5., 0., 5.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([
                55, 56, 57, 58, 59, 65, 66, 67, 68, 69, 75, 76, 77, 78, 79, 85, 86, 87, 88, 89, 95,
                96, 97, 98, 99
            ])
            .all(|(tile, i)| tile == i));
        // Iterating over top right tiles, but a bit shifted
        assert!(quad_tree
            .iter_tiles(Aabb::from_min_max(
                Vec3::new(2., 0., 2.),
                Vec3::new(7., 0., 7.)
            ))
            .map(|tile_ref| tile_ref.tile.tile_type)
            .zip([77, 78, 79, 87, 88, 89, 97, 98, 99])
            .all(|(tile, i)| tile == i));
    }
}
