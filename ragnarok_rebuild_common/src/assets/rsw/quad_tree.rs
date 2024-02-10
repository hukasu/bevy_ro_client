pub const QUAD_TREE_SIZE: usize = 1365;

#[derive(Debug)]
pub struct QuadTree {
    pub ranges: Box<[QuadTreeRange]>,
}

#[derive(Debug)]
pub struct QuadTreeRange {
    pub top: (f32, f32, f32),
    pub bottom: (f32, f32, f32),
    pub diameter: (f32, f32, f32),
    pub center: (f32, f32, f32),
}
