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
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
