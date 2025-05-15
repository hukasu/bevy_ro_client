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

struct SprUniform {
    uv_flip: u32,
    tint: vec4<f32>,
}

@group(2) @binding(0) var<uniform> spr_uniform: SprUniform;

#ifdef SPR_INDEXED_PIPELINE
@group(2) @binding(1) var spr_texture: texture_2d<u32>;
@group(2) @binding(2) var spr_palette: texture_1d<f32>;
#else ifdef SPR_TRUE_COLOR_PIPELINE
@group(2) @binding(1) var spr_texture: texture_2d<f32>;
@group(2) @binding(2) var spr_sampler: sampler;
#endif

@vertex
fn vertex(in: Vertex) -> VertexOutput {
    var vertex_output: VertexOutput;

    var dimensions = vec2<f32>(textureDimensions(spr_texture));
    let position = vec4<f32>(
        in.position.xy * dimensions,
        0.0,
        1.0
    );
    
    var world_from_local = mesh_functions::get_world_from_local(in.instance_index);
    vertex_output.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, position);
    vertex_output.position = position_world_to_clip(vertex_output.world_position.xyz);

#ifndef PREPASS_PIPELINE
    vertex_output.position += vec4(0., 0., 1. / 192., 0.);
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
#else
#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    vertex_output.world_normal = mesh_functions::mesh_normal_local_to_world(
        in.normal,
        in.instance_index
    );
#endif
#endif
    if spr_uniform.uv_flip == 1u {
        vertex_output.uv = vec2<f32>(1. - in.uv.x, in.uv.y);
    } else {
        vertex_output.uv = in.uv;
    }

    return vertex_output;
}
