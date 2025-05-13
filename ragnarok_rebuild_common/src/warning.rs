use std::fmt::Display;

/// Report warnings that might happen on assets.
///
/// Warnings refer to thigns that are not implemented currrently,
/// things that require deeper consideration on implementation,
/// or are plain weird values that don't work.
pub trait ReportWarning {
    fn report(&self) -> impl Display;
}
