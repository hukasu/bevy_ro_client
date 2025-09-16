use std::fmt::Display;

/// Report warnings that might happen on assets.
///
/// Warnings refer to things that are not implemented currently,
/// things that require deeper consideration on implementation,
/// or are plain weird values that don't work.
pub trait ReportWarning {
    fn report(&self) -> impl Display;
}
