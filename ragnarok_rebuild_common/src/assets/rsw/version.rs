#[derive(Debug, PartialEq, PartialOrd)]
pub struct Version(pub u8, pub u8, pub u32);

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Version(major, minor, build) = self;
        write!(f, "{major}.{minor}.{build}")
    }
}
