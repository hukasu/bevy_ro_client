use std::{collections::VecDeque, io::Read, ops::Deref};

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

#[derive(Debug, Clone)]
/// An Iterator-like object to crawl through a [QuadTree]
pub struct Crawler<'a> {
    quad_tree: &'a QuadTree,
    stack: VecDeque<usize>,
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
            stack: VecDeque::default(),
            depth: 0,
            index: 0,
        }
    }

    fn next_index(&mut self, direction: CrawlDirection) -> Option<&Range> {
        if self.depth >= QUAD_TREE_MAX_DEPTH {
            None
        } else {
            self.stack.push_back(self.index);

            let direction_addition = match direction {
                CrawlDirection::BottomLeft => 1,
                CrawlDirection::BottomRight => 2,
                CrawlDirection::TopLeft => 3,
                CrawlDirection::TopRight => 4,
            };

            let till_max_depth = QUAD_TREE_MAX_DEPTH - self.depth;
            let index_skip = (((4usize.pow(till_max_depth as u32) - 4) / 3)
                * (direction_addition - 1))
                + direction_addition;

            self.depth += 1;
            self.index += index_skip;

            Some(&self.quad_tree.ranges[self.index])
        }
    }

    pub fn parent(&mut self) -> Option<&Range> {
        if let Some(old) = self.stack.pop_back() {
            self.depth -= 1;
            self.index = old;
            Some(&self.quad_tree.ranges[self.index])
        } else {
            None
        }
    }

    pub fn top_left(&mut self) -> Option<&Range> {
        self.next_index(CrawlDirection::TopLeft)
    }

    pub fn top_right(&mut self) -> Option<&Range> {
        self.next_index(CrawlDirection::TopRight)
    }

    pub fn bottom_left(&mut self) -> Option<&Range> {
        self.next_index(CrawlDirection::BottomLeft)
    }

    pub fn bottom_right(&mut self) -> Option<&Range> {
        self.next_index(CrawlDirection::BottomRight)
    }
}

impl Deref for Crawler<'_> {
    type Target = Range;

    fn deref(&self) -> &Self::Target {
        &self.quad_tree.ranges[self.index]
    }
}
