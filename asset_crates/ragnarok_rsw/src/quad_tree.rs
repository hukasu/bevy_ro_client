use std::{io::Read, ops::Deref};

use ragnarok_rebuild_common::reader_ext::ReaderExt;

pub const QUAD_TREE_MAX_DEPTH: usize = 5;
pub const QUAD_TREE_SIZE: usize = 1365;

#[derive(Debug)]
/// A QuadTree
///
/// # Note
/// Ragnarok uses a `Left-handed Y-down` coordinate system, which means the top of the [QuadTree] has a
/// `negative Y` and the bottom has a `positive Y`.
pub struct QuadTree {
    pub ranges: Box<[Range]>,
}

impl QuadTree {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<QuadTree, super::Error> {
        let ranges = (0..QUAD_TREE_SIZE)
            .map(|_| {
                let bottom = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                let top = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                let radius = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                let center = [
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                ];
                Ok(Range {
                    top,
                    bottom,
                    radius,
                    center,
                })
            })
            .collect::<Result<Box<[Range]>, super::Error>>()?;
        Ok(QuadTree { ranges })
    }

    pub fn crawl(&'_ self) -> Crawler<'_> {
        Crawler::new(self)
    }
}

#[derive(Debug, Default, Clone, Copy)]
/// A node in the [QuadTree]
///
/// # Note
/// Ragnarok uses a `Left-handed Y-down` coordinate system, and the name of the
/// members reflect that.
pub struct Range {
    /// The top(Y) bottom-left(XY) point of the bounding box.
    /// Due to the coordinate system, the Y of the bottom > top.
    pub top: [f32; 3],
    /// The bottom(Y) top-right(XY) point of the bounding box.
    /// Due to the coordinate system, the Y of the bottom > top.
    pub bottom: [f32; 3],
    /// The radius of the bounding box, each component represents the
    /// distance between the center and the edge axis aligned.
    pub radius: [f32; 3],
    // The center of the bounding box.
    pub center: [f32; 3],
}

#[derive(Debug, Clone, Copy)]
/// An Iterator-like object to crawl through a [QuadTree]
pub struct Crawler<'a> {
    quad_tree: &'a QuadTree,
    directions: usize,
    depth: usize,
    index: usize,
}

#[derive(Debug, PartialEq)]
/// The directions that a [Crawler] can move through the [QuadTree]
enum CrawlDirection {
    BottomLeft,
    BottomRight,
    TopLeft,
    TopRight,
}

impl<'a> Crawler<'a> {
    fn new(quad_tree: &'a QuadTree) -> Self {
        Self {
            quad_tree,
            directions: 0,
            depth: 0,
            index: 0,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.depth >= QUAD_TREE_MAX_DEPTH
    }

    fn index_skip(depth: usize, direction: usize) -> usize {
        let till_max_depth = QUAD_TREE_MAX_DEPTH - depth;
        (((4usize.pow(till_max_depth as u32) - 4) / 3) * (direction - 1)) + direction
    }

    fn next_index(&self, direction: CrawlDirection) -> Option<Crawler<'a>> {
        if self.is_leaf() {
            None
        } else {
            let direction_addition = match direction {
                CrawlDirection::BottomLeft => 1,
                CrawlDirection::BottomRight => 2,
                CrawlDirection::TopLeft => 3,
                CrawlDirection::TopRight => 4,
            };

            let index_skip = Self::index_skip(self.depth, direction_addition);

            let next_depth = self.depth + 1;
            let next_index = self.index + index_skip;

            Some(Crawler {
                quad_tree: self.quad_tree,
                directions: (self.directions << 2) + (direction_addition - 1),
                depth: next_depth,
                index: next_index,
            })
        }
    }

    pub fn parent(&self) -> Option<Crawler<'a>> {
        if self.depth == 0 {
            None
        } else {
            let direction_addition = (self.directions & 3) + 1;

            let skipped_indexes = Self::index_skip(self.depth - 1, direction_addition);

            let next_index = self.index - skipped_indexes;

            Some(Crawler {
                quad_tree: self.quad_tree,
                directions: self.directions >> 2,
                depth: self.depth - 1,
                index: next_index,
            })
        }
    }

    pub fn top_left(&self) -> Option<Crawler<'a>> {
        self.next_index(CrawlDirection::TopLeft)
    }

    pub fn top_right(&self) -> Option<Crawler<'a>> {
        self.next_index(CrawlDirection::TopRight)
    }

    pub fn bottom_left(&self) -> Option<Crawler<'a>> {
        self.next_index(CrawlDirection::BottomLeft)
    }

    pub fn bottom_right(&self) -> Option<Crawler<'a>> {
        self.next_index(CrawlDirection::BottomRight)
    }
}

impl Deref for Crawler<'_> {
    type Target = Range;

    fn deref(&self) -> &Self::Target {
        &self.quad_tree.ranges[self.index]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[expect(clippy::unwrap_used, reason = "This is a test")]
    fn test_parent() {
        fn recursive(crawler: &Crawler) {
            let current_index = crawler.index;
            let current_depth = crawler.depth;

            if !crawler.is_leaf() {
                let bl = crawler.bottom_left().unwrap();
                assert_eq!(bl.parent().unwrap().index, current_index);
                assert_eq!(bl.parent().unwrap().depth, current_depth);
                recursive(&bl);

                let br = crawler.bottom_right().unwrap();
                assert_eq!(br.parent().unwrap().index, current_index);
                assert_eq!(br.parent().unwrap().depth, current_depth);
                recursive(&br);

                let tl = crawler.top_left().unwrap();
                assert_eq!(tl.parent().unwrap().index, current_index);
                assert_eq!(tl.parent().unwrap().depth, current_depth);
                recursive(&tl);

                let tr = crawler.top_right().unwrap();
                assert_eq!(tr.parent().unwrap().index, current_index);
                assert_eq!(tr.parent().unwrap().depth, current_depth);
                recursive(&tr);
            }
        }

        let quad_tree = QuadTree {
            ranges: vec![
                Range {
                    center: [0., 0., 0.],
                    radius: [0., 0., 0.],
                    bottom: [0., 0., 0.],
                    top: [0., 0., 0.],
                };
                QUAD_TREE_SIZE
            ]
            .into_boxed_slice(),
        };

        let crawler = quad_tree.crawl();
        recursive(&crawler);
    }
}
