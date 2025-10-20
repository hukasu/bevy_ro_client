use std::{fmt::Display, ops::Deref};

use ragnarok_rebuild_common::warning::ReportWarning;

use crate::Gnd;

impl ReportWarning for Gnd {
    fn report(&self) -> impl Display {
        GndWarning(self)
    }
}

struct GndWarning<'a>(&'a Gnd);

impl Deref for GndWarning<'_> {
    type Target = Gnd;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Display for GndWarning<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.report_dimensions(f)?;
        Ok(())
    }
}

impl GndWarning<'_> {
    fn report_dimensions(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.width.is_multiple_of(2) {
            writeln!(f, "width is not multiple of 2. {}", self.width)?;
        }
        if !self.height.is_multiple_of(2) {
            writeln!(f, "height is not multiple of 2. {}", self.width)?;
        }
        Ok(())
    }
}
