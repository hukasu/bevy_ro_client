use std::{fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{
    act::{Act, AnimationClip, AnimationFrame, SpriteLayer},
    grf::GRF,
};

fn main() {
    let Ok(grf) = GRF::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for act_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("act"))
            }
            None => false,
        })
    {
        let Ok(act_content) = grf
            .read_file(act_filename)
            .inspect_err(|err| println!("{act_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(act) = Act::from_reader(&mut Cursor::new(act_content))
            .inspect_err(|err| println!("{act_filename:?}: {err}"))
        else {
            continue;
        };

        if let Some(act_debug) = debug_act(&act) {
            println!("{:?}", act_filename);
            println!("{}", act_debug);
        }
    }
}

fn debug_act(act: &Act) -> Option<String> {
    let header = || format!("\t{:?}\n", act.version);
    let mut debug = None;

    if act.animation_clips.is_empty() {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\thas no animation clips.").unwrap();
    }

    for (i, clip) in act.animation_clips.iter().enumerate() {
        if let Some(animation_clip_debug) = debug_animation_clip(clip, i, act) {
            let debug_ref = debug.get_or_insert_with(header);
            write!(debug_ref, "{}", animation_clip_debug).unwrap();
        }
    }

    debug
}

fn debug_animation_clip(clip: &AnimationClip, clip_index: usize, act: &Act) -> Option<String> {
    let header = || format!("\t\tClip {:?}\n", clip_index);
    let mut debug = None;

    for (i, frame) in clip.animation_frames.iter().enumerate() {
        if let Some(animation_frame_debug) = debug_animation_frame(frame, i, act) {
            let debug_ref = debug.get_or_insert_with(header);
            write!(debug_ref, "{}", animation_frame_debug).unwrap();
        }
    }

    debug
}

fn debug_animation_frame(frame: &AnimationFrame, frame_index: usize, act: &Act) -> Option<String> {
    const INDENT: &str = "\t\t\t";
    let header = || format!("{INDENT}Frame {:?}\n", frame_index);
    let mut debug = None;

    if frame.animation_event_id != -1
        && frame.animation_event_id.unsigned_abs() as usize >= act.animation_events.len()
    {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "{INDENT}\tFrame {} has event outside of bounds. Act has {} events but frame event id was {}.",
            frame_index,
            act.animation_events.len(),
            frame.animation_event_id
        )
        .unwrap();
    }

    for (i, layer) in frame.sprite_layers.iter().enumerate() {
        if let Some(animation_frame_debug) = debug_animation_layer(layer, i) {
            let debug_ref = debug.get_or_insert_with(header);
            write!(debug_ref, "{}", animation_frame_debug).unwrap();
        }
    }

    debug
}

fn debug_animation_layer(layer: &SpriteLayer, frame_index: usize) -> Option<String> {
    const INDENT: &str = "\t\t\t\t";
    let header = || format!("{INDENT}Layer {:?}\n", frame_index);
    let mut debug = None;

    match layer.image_type_id {
        0 | 1 => (),
        id => {
            let debug_ref = debug.get_or_insert_with(header);
            writeln!(
                debug_ref,
                "{INDENT}\tLayer has unknown image type id ({}).",
                id
            )
            .unwrap();
        }
    }

    if layer.spritesheet_cell_index < -1 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "{INDENT}\tLayer has negative spritesheet index ({}).",
            layer.spritesheet_cell_index
        )
        .unwrap();
    }

    debug
}
