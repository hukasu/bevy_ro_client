#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    pub alpha: T,
}

pub type UColor = Color<u8>;
pub type FColor = Color<f32>;

impl From<UColor> for FColor {
    fn from(value: UColor) -> Self {
        Self {
            red: value.red as f32 / 255.,
            green: value.green as f32 / 255.,
            blue: value.blue as f32 / 255.,
            alpha: value.alpha as f32 / 255.,
        }
    }
}
