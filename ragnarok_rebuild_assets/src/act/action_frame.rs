use crate::reader_ext::ReaderExt;

#[derive(Debug)]
pub struct ActionFrame {
    attack_range: [u32; 4],
    fit_range: [u32; 4],
    layer_count: u32,
    layers: Box<[super::FrameLayer]>,
    event_id: i32,
    anchor_point_count: u32,
    anchor_points: Box<[super::AnchorPoint]>,
}

impl ActionFrame {
    pub fn from_bytes(bytes: &mut &[u8]) -> Result<Self, std::io::Error> {
        let mut attack_range = [0; 4];
        for r in attack_range.iter_mut() {
            *r = bytes.read_le_u32()?;
        }
        let mut fit_range = [0; 4];
        for r in fit_range.iter_mut() {
            *r = bytes.read_le_u32()?;
        }

        let layer_count = bytes.read_le_u32()?;
        let layers = (0..layer_count)
            .map(|_| super::FrameLayer::from_bytes(bytes))
            .collect::<Result<_, _>>()?;

        let event_id = bytes.read_le_i32()?;

        let anchor_point_count = bytes.read_le_u32()?;
        let anchor_points = (0..anchor_point_count)
            .map(|_| super::AnchorPoint::from_bytes(bytes))
            .collect::<Result<_, _>>()?;

        Ok(Self {
            attack_range,
            fit_range,
            layer_count,
            layers,
            event_id,
            anchor_point_count,
            anchor_points,
        })
    }
}
