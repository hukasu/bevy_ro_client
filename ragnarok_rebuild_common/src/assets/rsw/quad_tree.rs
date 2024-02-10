use std::{collections::VecDeque, io::Read, ops::Deref};

use crate::reader_ext::ReaderExt;

pub const QUAD_TREE_MAX_DEPTH: usize = 5;
pub const QUAD_TREE_SIZE: usize = 1365;

#[derive(Debug)]
pub struct QuadTree {
    pub ranges: Box<[Range]>,
}

impl QuadTree {
    pub fn from_reader(mut reader: &mut dyn Read) -> Result<QuadTree, super::Error> {
        let ranges = (0..QUAD_TREE_SIZE)
            .map(|_| {
                let top = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                let bottom = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                let diameter = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                let center = (
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                    reader.read_le_f32()?,
                );
                Ok(Range {
                    top,
                    bottom,
                    diameter,
                    center,
                })
            })
            .collect::<Result<Box<[Range]>, super::Error>>()?;
        Ok(QuadTree { ranges })
    }

    pub fn crawl(&self) -> Crawler {
        Crawler::new(self)
    }
}

#[derive(Debug, Default)]
pub struct Range {
    pub top: (f32, f32, f32),
    pub bottom: (f32, f32, f32),
    pub diameter: (f32, f32, f32),
    pub center: (f32, f32, f32),
}

#[derive(Debug, Clone)]
pub struct Crawler<'a> {
    quad_tree: &'a QuadTree,
    stack: VecDeque<usize>,
    depth: usize,
    index: usize,
}

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
        if self.depth >= 5 {
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

impl<'a> Deref for Crawler<'a> {
    type Target = Range;

    fn deref(&self) -> &Self::Target {
        &self.quad_tree.ranges[self.index]
    }
}
