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
}
