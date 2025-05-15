use std::{fmt::Display, ops::Deref};

use ragnarok_rebuild_common::warning::ReportWarning;

use crate::Pal;

impl ReportWarning for Pal {
    fn report(&self) -> impl Display {
        PalReport(self)
    }
}

struct PalReport<'a>(&'a Pal);

impl Display for PalReport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.colors[0].alpha != 0 {
            writeln!(
                f,
                "Key color has non-zero alpha of {}.",
                self.colors[0].alpha
            )?;
        }

        for (i, color) in self.colors[1..].iter().enumerate() {
            if color.alpha > 0 && color.alpha < 255 {
                writeln!(f, "Color {} has alpha of {}.", i, color.alpha)?;
            }
        }

        Ok(())
    }
}

impl Deref for PalReport<'_> {
    type Target = Pal;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
