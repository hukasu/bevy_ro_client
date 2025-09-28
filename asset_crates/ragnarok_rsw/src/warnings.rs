use std::{collections::BTreeSet, fmt::Display, ops::Deref};

use ragnarok_rebuild_common::warning::ReportWarning;

use crate::{Light, Model, Rsw};

impl ReportWarning for Rsw {
    fn report(&self) -> impl Display {
        RswWarning(self)
    }
}

struct RswWarning<'a>(&'a Rsw);

impl RswWarning<'_> {
    fn report_presence_rsm2(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self
            .models
            .iter()
            .any(|model| model.filename.ends_with(".rsm2"))
        {
            writeln!(f, "has RSM2 models.")?;
        }
        Ok(())
    }

    fn report_models(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for model in self.models.iter() {
            let model_warnings = model.report().to_string();
            if !model_warnings.is_empty() {
                writeln!(f, "Model {}", model.name)?;
                write!(f, "{model_warnings}")?;
            }
        }
        Ok(())
    }

    fn report_lights(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for light in self.lights.iter() {
            let light_warnings = light.report().to_string();
            if !light_warnings.is_empty() {
                writeln!(f, "Light {}", light.name)?;
                write!(f, "{light_warnings}")?;
            }
        }
        Ok(())
    }

    fn report_overlapping_lights(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const THRESHOLD: f32 = 1.;

        let mut light_positions: Vec<&Light> = Vec::new();
        for light in self.lights.iter() {
            if let Some(repeated) = light_positions.iter().find(|other| {
                ((other.position[0] - light.position[0]).powi(2)
                    + (other.position[1] - light.position[1]).powi(2)
                    + (other.position[2] - light.position[2]).powi(2))
                .sqrt()
                    < THRESHOLD
            }) {
                writeln!(
                    f,
                    "has a repeated light at {:?}. ({}, {})",
                    light.position, light.name, repeated.name
                )?;
            } else {
                light_positions.push(light);
            }
        }
        Ok(())
    }

    fn report_duplicate_light_name(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut light_names = BTreeSet::new();
        for light in self.lights.iter() {
            if light_names.contains(&light.name) {
                writeln!(f, "has a repeated light name. ({})", light.name)?;
            } else {
                light_names.insert(&light.name);
            }
        }
        Ok(())
    }
}

impl Deref for RswWarning<'_> {
    type Target = Rsw;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Display for RswWarning<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.report_presence_rsm2(f)?;
        self.report_overlapping_lights(f)?;
        self.report_duplicate_light_name(f)?;
        self.report_models(f)?;
        self.report_lights(f)?;
        Ok(())
    }
}

impl ReportWarning for Model {
    fn report(&self) -> impl Display {
        RswModelWarning(self)
    }
}
struct RswModelWarning<'a>(&'a Model);

impl RswModelWarning<'_> {
    fn report_filename(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.filename.is_empty() {
            writeln!(f, "has empty filename.")?;
        } else if self.filename.len() > 75 {
            writeln!(f, "has long filename. ({})", self.filename.len())?;
        }
        Ok(())
    }

    fn report_node_name(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Node name is frequently empty, so no check for it
        if self.node_name.len() > 75 {
            writeln!(f, "has long node name. ({})", self.node_name.len())?;
        }
        Ok(())
    }

    fn report_flags(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.flag != 0 {
            writeln!(f, "has non-zero flags. ({})", self.flag)?;
        }
        Ok(())
    }
}

impl Deref for RswModelWarning<'_> {
    type Target = Model;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Display for RswModelWarning<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.report_filename(f)?;
        self.report_node_name(f)?;
        self.report_flags(f)?;
        Ok(())
    }
}

impl ReportWarning for Light {
    fn report(&self) -> impl Display {
        RswLightWarning(self)
    }
}
struct RswLightWarning<'a>(&'a Light);

impl RswLightWarning<'_> {
    fn report_blownout_color(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self
            .color
            .iter()
            .any(|channel| *channel < 0. || *channel > 1.)
        {
            writeln!(f, "has unnormalized color {:?}.", self.color)?;
        }
        Ok(())
    }
}

impl Deref for RswLightWarning<'_> {
    type Target = Light;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl Display for RswLightWarning<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.report_blownout_color(f)?;
        Ok(())
    }
}
