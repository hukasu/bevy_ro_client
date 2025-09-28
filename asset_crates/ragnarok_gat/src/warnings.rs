use std::{fmt::Display, ops::Deref};

use ragnarok_rebuild_common::warning::ReportWarning;

use crate::Gat;

const AXIS_LIMIT: u32 = 416;

impl ReportWarning for Gat {
    fn report(&self) -> impl Display {
        GatReport(self)
    }
}

struct GatReport<'a>(&'a Gat);

impl GatReport<'_> {
    fn report_axis(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.width > AXIS_LIMIT || self.height > AXIS_LIMIT {
            writeln!(
                f,
                "has width or height over {AXIS_LIMIT}. ({}x{})",
                self.width, self.height
            )?;
        }
        Ok(())
    }
}

impl Display for GatReport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.report_axis(f)?;
        Ok(())
    }
}

impl Deref for GatReport<'_> {
    type Target = Gat;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
