use std::{collections::BTreeSet, fmt::Write, io::Cursor, path::Path};

use ragnarok_rebuild_assets::{
    grf::GRF,
    rsm::{
        mesh::{Face, Mesh},
        RSM,
    },
};

fn main() {
    let Ok(grf) = GRF::new(Path::new("data.grf")).inspect_err(|err| eprintln!("{err}")) else {
        return;
    };

    for rsm_filename in grf
        .iter_filenames()
        .filter(|filename| match filename.extension() {
            Some(ext) => {
                matches!(ext.to_str(), Some("rsm") | Some("rsm2"))
            }
            None => false,
        })
    {
        let Ok(rsm_content) = grf
            .read_file(rsm_filename)
            .inspect_err(|err| println!("{rsm_filename:?}: {err}"))
        else {
            continue;
        };
        let Ok(rsm) = RSM::from_reader(&mut Cursor::new(rsm_content))
            .inspect_err(|err| println!("{rsm_filename:?}: {err}"))
        else {
            continue;
        };

        if let Some(rsm_debug) = debug_rsm(&rsm) {
            println!("{:?}", rsm_filename);
            println!("{}", rsm_debug);
        }
    }
}

fn debug_rsm(rsm: &RSM) -> Option<String> {
    let header = || format!("\t{:?}\n", rsm.version);
    let mut debug = None;

    if rsm.root_meshes.is_empty() {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\thas no root meshes.").unwrap();
    }

    if rsm.volume_boxes.len() != 0 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\thas volume boxes. ({:?})", rsm.volume_boxes).unwrap();
    }

    for mesh in rsm.meshes.iter() {
        if let Some(mesh_debug) = debug_mesh(mesh) {
            let debug_ref = debug.get_or_insert_with(header);
            write!(debug_ref, "{}", mesh_debug).unwrap();
        }
    }

    debug
}

fn debug_mesh(mesh: &Mesh) -> Option<String> {
    let header = || format!("\tMesh \"{}\"\n", mesh.name);
    let mut debug = None;

    if mesh.name.is_empty() {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\thas empty name.",).unwrap();
    }

    if mesh.name == mesh.parent_name {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "\t\thas name equal to parent_name. (\"{}\")",
            mesh.name
        )
        .unwrap();
    }

    for (i, face) in mesh.faces.iter().enumerate() {
        if let Some(face_debug) = debug_face(face, i) {
            let debug_ref = debug.get_or_insert_with(header);
            write!(debug_ref, "{}", face_debug).unwrap();
        }
    }

    if let Some(face_debug) = check_unused_vertices(mesh) {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\t{}", face_debug).unwrap();
    }

    debug
}

fn debug_face(face: &Face, index: usize) -> Option<String> {
    let header = || format!("\t\tFace {}\n", index);

    let mut debug = None;

    if face.smoothing_group.len() > 3 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(
            debug_ref,
            "\t\t\t has more {} smoothing groups. ({:?})",
            face.smoothing_group.len(),
            face.smoothing_group
        )
        .unwrap();
    } else if face.smoothing_group.len() == 0 {
        let debug_ref = debug.get_or_insert_with(header);
        writeln!(debug_ref, "\t\t\t has more no smoothing groups.",).unwrap();
    }

    debug
}

fn check_unused_vertices(mesh: &Mesh) -> Option<String> {
    let mut used_vertices = BTreeSet::new();

    for face in mesh.faces.iter() {
        used_vertices.insert(face.vertices[0]);
        used_vertices.insert(face.vertices[1]);
        used_vertices.insert(face.vertices[2]);
    }

    if used_vertices.len() != mesh.vertices.len() {
        Some(format!(
            "has unused vertices. Has {}, used {}.",
            mesh.vertices.len(),
            used_vertices.len()
        ))
    } else {
        None
    }
}
