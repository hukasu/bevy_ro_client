#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{Vertex, VertexOutput},
}
#else
#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput},
}
#endif

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;

    let position = vec4<f32>(
        in.position,
        1.0
    );
    
    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz);
    
#ifdef MESH_PIPELINE
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
#endif
    vertex_output.uv = in.uv;

#ifdef DEPTH_CLAMP_ORTHO
    vertex_output.clip_position_unclamped = vertex_output.position;
    vertex_output.position.z = min(vertex_output.position.z, 1.0);
#endif

    return vertex_output;
}
