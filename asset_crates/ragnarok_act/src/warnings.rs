use std::{fmt::Display, ops::Deref};

use ragnarok_rebuild_common::warning::ReportWarning;

use crate::Act;

impl ReportWarning for Act {
    fn report(&self) -> impl std::fmt::Display {
        ActReport(self)
    }
}

struct ActReport<'a>(&'a Act);

impl Display for ActReport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.animation_clips.is_empty() {
            writeln!(f, "Has no clips.")?;
        }

        for (i, clip) in self.animation_clips.iter().enumerate() {
            if clip.animation_frames.is_empty() {
                writeln!(f, "Clip{i} has no frames.")?;
            }

            for (j, frame) in clip.animation_frames.iter().enumerate() {
                if frame.sprite_layers.is_empty() {
                    writeln!(f, "Clip{i}/Frame{j} has no layers.")?;
                }

                if frame.animation_event_id >= 0 {
                    if let Ok(index) = usize::try_from(frame.animation_event_id) {
                        if index >= self.animation_events.len() {
                            writeln!(
                                f,
                                "Clip{i}/Frame{j} accesses event {} out of bounds.",
                                frame.animation_event_id
                            )?;
                        }
                    } else {
                        writeln!(
                            f,
                            "Clip{i}/Frame{j} has unaddressable event {}.",
                            frame.animation_event_id
                        )?;
                    }
                } else if frame.animation_event_id < 0 && frame.animation_event_id != -1 {
                    writeln!(
                        f,
                        "Clip{i}/Frame{j} has negative event {}.",
                        frame.animation_event_id
                    )?;
                }

                for (k, layer) in frame.sprite_layers.iter().enumerate() {
                    if layer.spritesheet_cell_index < -1 {
                        writeln!(
                            f,
                            "Clip{i}/Frame{j}/Layer{k} has negative spritesheet cell {}.",
                            layer.spritesheet_cell_index
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

impl Deref for ActReport<'_> {
    type Target = Act;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
