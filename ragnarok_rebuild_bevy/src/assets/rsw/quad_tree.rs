use std::{borrow::Borrow, ops::Index};

use bevy::{math::Vec3, reflect::Reflect, render::primitives::Aabb};

use ragnarok_rebuild_assets::rsw;

#[derive(Debug, Reflect)]
/// Gat's tile QuadTree
pub struct QuadTree {
    ranges: Vec<Aabb>,
}

impl QuadTree {
    pub const MAX_DEPTH: usize = 5;
    pub const SIZE: usize = 1365;

    /// Iterate over all nodes in the [`QuadTree`]
    ///
    /// Since [`QuadTree`]s is fixed size, this method does not
    /// need a reference to a [`QuadTree`].
    pub fn iter_indexes(&self) -> impl Iterator<Item = QuadTreeIndex> {
        QuadTreeIter {
            stack: vec![QuadTreeIndex::default()],
        }
    }

    // pub fn iter_tiles(&self, quad_tree_index: QuadTreeIndex) -> impl Iterator<Item = &Tile> {
    //     let aabb = self[quad_tree_index];
    //     Self::get_tiles_from_aabb(&aabb, &self.tiles, self.tiles_width, self.tiles_height)
    // }

    // fn get_tiles_from_aabb<'a>(
    //     aabb: &Aabb,
    //     tiles: &'a [Tile],
    //     width: u32,
    //     height: u32,
    // ) -> impl Iterator<Item = &'a Tile> + Clone {
    //     let min = (aabb.min() + Vec3A::new((width / 2) as f32, 0., (height / 2) as f32)).as_uvec3();
    //     let max = (aabb.max() + Vec3A::new((width / 2) as f32, 0., (height / 2) as f32)).as_uvec3();

    //     ((min.z as usize)..(max.z as usize)).flat_map(move |z| {
    //         ((min.x as usize)..(max.x as usize)).map(move |x| &tiles[x + z * width as usize])
    //     })
    // }
}

impl Index<QuadTreeIndex> for QuadTree {
    type Output = Aabb;

    fn index(&self, index: QuadTreeIndex) -> &Self::Output {
        &self.ranges[index.index]
    }
}

impl<T: Borrow<rsw::QuadTree>> From<T> for QuadTree {
    fn from(value: T) -> Self {
        Self {
            ranges: value
                .borrow()
                .ranges
                .iter()
                .map(|range| {
                    Aabb::from_min_max(Vec3::from_array(range.bottom), Vec3::from_array(range.top))
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuadTreeIter {
    stack: Vec<QuadTreeIndex>,
}

impl Iterator for QuadTreeIter {
    type Item = QuadTreeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::expect_used)]
        self.stack.pop().inspect(|head| {
            if !head.is_leaf() {
                self.stack.push(
                    head.top_right()
                        .expect("Should have top right if not leaf."),
                );
                self.stack
                    .push(head.top_left().expect("Should have top left if not leaf."));
                self.stack.push(
                    head.bottom_right()
                        .expect("Should have bottom right if not leaf."),
                );
                self.stack.push(
                    head.bottom_left()
                        .expect("Should have bottom left if not leaf."),
                );
            };
        })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct QuadTreeIndex {
    index: usize,
    depth: usize,
    stack: [usize; QuadTree::MAX_DEPTH],
}

impl QuadTreeIndex {
    pub fn parent(&self) -> Option<QuadTreeIndex> {
        if self.index == 0 {
            None
        } else {
            let Some(parent) = self.stack.last() else {
                unreachable!("Stack shouldn't be empty if index not equals 0.");
            };

            let mut stack = self.stack;
            stack[self.depth - 1] = 0;

            Some(Self {
                index: *parent,
                depth: self.depth - 1,
                stack,
            })
        }
    }

    fn next_index(&self, direction: usize) -> Option<QuadTreeIndex> {
        if self.depth >= QuadTree::MAX_DEPTH {
            None
        } else {
            let till_max_depth = QuadTree::MAX_DEPTH - self.depth;
            let index_skip =
                (((4usize.pow(till_max_depth as u32) - 4) / 3) * (direction - 1)) + direction;

            let mut stack = self.stack;
            stack[self.depth] = self.index;

            Some(Self {
                index: self.index + index_skip,
                depth: self.depth + 1,
                stack,
            })
        }
    }

    pub fn bottom_left(&self) -> Option<QuadTreeIndex> {
        self.next_index(1)
    }

    pub fn bottom_right(&self) -> Option<QuadTreeIndex> {
        self.next_index(2)
    }

    pub fn top_left(&self) -> Option<QuadTreeIndex> {
        self.next_index(3)
    }

    pub fn top_right(&self) -> Option<QuadTreeIndex> {
        self.next_index(4)
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn is_leaf(&self) -> bool {
        self.depth == QuadTree::MAX_DEPTH
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::{QuadTree, QuadTreeIndex};

    #[test]
    fn in_order_iteration() {
        let indexes = QuadTree { ranges: vec![] }
            .iter_indexes()
            .map(|node| node.index)
            .collect::<Vec<_>>();
        let expected: [usize; QuadTree::SIZE] = std::array::from_fn(|i| i);
        assert_eq!(&indexes, &expected);
    }

    #[test]
    fn test_parent() {
        let index = QuadTreeIndex {
            index: 0,
            depth: 0,
            stack: [0; 5],
        };
        let bottom_left = index.bottom_left().unwrap();
        let bottom_left_parent = bottom_left.parent().unwrap();
        assert_eq!(bottom_left_parent, index);
        let bottom_right = index.bottom_right().unwrap();
        let bottom_right_parent = bottom_right.parent().unwrap();
        assert_eq!(bottom_right_parent, index);
        let top_left = index.top_left().unwrap();
        let top_left_parent = top_left.parent().unwrap();
        assert_eq!(top_left_parent, index);
        let top_right = index.top_right().unwrap();
        let top_right_parent = top_right.parent().unwrap();
        assert_eq!(top_right_parent, index);
    }

    // #[test]
    // fn test_iterate_tiles() {
    //     let quad_tree = QuadTree {
    //         ranges: vec![
    //             // Root
    //             Aabb::from_min_max(Vec3::new(-5., 0., -5.), Vec3::new(5., 0., 5.)),
    //             // Bottom Left
    //             Aabb::from_min_max(Vec3::new(-5., 0., -5.), Vec3::new(0., 0., 0.)),
    //             // Bottom Right
    //             Aabb::from_min_max(Vec3::new(0., 0., -5.), Vec3::new(5., 0., 0.)),
    //             // Top Left
    //             Aabb::from_min_max(Vec3::new(-5., 0., 0.), Vec3::new(0., 0., 5.)),
    //             // Top Right
    //             Aabb::from_min_max(Vec3::new(0., 0., 0.), Vec3::new(5., 0., 5.)),
    //         ],
    //         tiles_width: 10,
    //         tiles_height: 10,
    //         tiles: (0..100)
    //             .map(|i| Tile {
    //                 bottom_left: 0.,
    //                 bottom_right: 0.,
    //                 top_left: 0.,
    //                 top_right: 0.,
    //                 tile_type: i,
    //                 is_water_tile: false,
    //             })
    //             .collect(),
    //     };
    //     // Iterating from the root should return all in order
    //     // Using hardcoded index since the full tree is not built
    //     assert!((0..100)
    //         .zip(quad_tree.iter_tiles(QuadTreeIndex::default()))
    //         .all(|(i, tile)| tile.tile_type == i));
    //     // Iterating over lower lefy tiles
    //     // Using hardcoded index since the full tree is not built
    //     assert!(quad_tree
    //         .iter_tiles(QuadTreeIndex {
    //             index: 1,
    //             ..Default::default()
    //         })
    //         .map(|tile| tile.tile_type)
    //         .zip([
    //             0, 1, 2, 3, 4, 10, 11, 12, 13, 14, 20, 21, 22, 23, 24, 30, 31, 32, 33, 34, 40, 41,
    //             42, 43, 44,
    //         ])
    //         .all(|(tile, i)| tile == i));
    //     // Iterating over lower right tiles
    //     // Using hardcoded index since the full tree is not built
    //     assert!(quad_tree
    //         .iter_tiles(QuadTreeIndex {
    //             index: 2,
    //             ..Default::default()
    //         })
    //         .map(|tile| tile.tile_type)
    //         .zip([
    //             5, 6, 7, 8, 9, 15, 16, 17, 18, 19, 25, 26, 27, 28, 29, 35, 36, 37, 38, 39, 45, 46,
    //             47, 48, 49
    //         ])
    //         .all(|(tile, i)| tile == i));
    //     // Iterating over top left tiles
    //     // Using hardcoded index since the full tree is not built
    //     assert!(quad_tree
    //         .iter_tiles(QuadTreeIndex {
    //             index: 3,
    //             ..Default::default()
    //         })
    //         .map(|tile| tile.tile_type)
    //         .zip([
    //             50, 51, 52, 53, 54, 60, 61, 62, 63, 64, 70, 71, 72, 73, 74, 80, 81, 82, 83, 84, 90,
    //             91, 92, 93, 94
    //         ])
    //         .all(|(tile, i)| tile == i));
    //     // Iterating over top right tiles
    //     // Using hardcoded index since the full tree is not built
    //     assert!(quad_tree
    //         .iter_tiles(QuadTreeIndex {
    //             index: 4,
    //             ..Default::default()
    //         })
    //         .map(|tile| tile.tile_type)
    //         .zip([
    //             55, 56, 57, 58, 59, 65, 66, 67, 68, 69, 75, 76, 77, 78, 79, 85, 86, 87, 88, 89, 95,
    //             96, 97, 98, 99
    //         ])
    //         .all(|(tile, i)| tile == i));
    // }
}
