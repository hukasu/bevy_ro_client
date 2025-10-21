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
        self.report_surfaces(f)?;
        self.report_east_edge(f)?;
        self.report_north_edge(f)?;
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

    fn report_surfaces(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, surface) in self.surfaces.iter().enumerate() {
            if usize::from(surface.texture_id) >= self.textures.len() {
                writeln!(f, "Surface {i} has invalid texture. {}", surface.texture_id)?;
            }
        }
        Ok(())
    }

    fn report_east_edge(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = self.width - 1;
        for z in 0..self.height {
            let Ok(index) = usize::try_from(x + z * self.width) else {
                unreachable!("Index must fit in usize.");
            };
            let cube = &self.ground_mesh_cubes[index];
            if cube.east_facing_surface != -1 {
                writeln!(
                    f,
                    "Cube {x}/{z} has east face. ({})",
                    cube.east_facing_surface
                )?;
            }
        }
        Ok(())
    }

    fn report_north_edge(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let z = self.height - 1;
        for x in 0..self.width {
            let Ok(index) = usize::try_from(x + z * self.width) else {
                unreachable!("Index must fit in usize.");
            };
            let cube = &self.ground_mesh_cubes[index];
            if cube.north_facing_surface != -1 {
                writeln!(
                    f,
                    "Cube {x}/{z} has north face. ({})",
                    cube.north_facing_surface
                )?;
            }
        }
        Ok(())
    }
}
