#import bevy_pbr::{
    mesh_functions,
    mesh_bindings::mesh,
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

struct SprUvs {
    min: vec2<f32>,
    max: vec2<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> spr_uniform: SprUniform;

#ifdef SPR_INDEXED_PIPELINE
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var spr_texture: texture_2d<u32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var spr_palette: texture_1d<f32>;
#else ifdef SPR_TRUE_COLOR_PIPELINE
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var spr_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var spr_sampler: sampler;
#endif

@group(#{MATERIAL_BIND_GROUP}) @binding(3) var<storage> spr_uvs: array<SprUvs>;

@vertex
fn vertex(
    in: Vertex,
#ifndef MORPH_TARGETS
    @builtin(vertex_index) vertex_index: u32,
#endif
) -> VertexOutput {
    var vertex_output: VertexOutput;

    let first_vertex_index = mesh[in.instance_index].first_vertex_index;
#ifndef MORPH_TARGETS
    let index = vertex_index - first_vertex_index;
#else
    let index = in.vertex_index - first_vertex_index;
#endif

#ifdef SPR_INDEXED_PIPELINE
    let spr_uv = spr_uvs[0];
    var uv: vec2<f32>;
    if index == 0 {
        uv = spr_uv.min;
    } else if index == 1 {
        uv = vec2(spr_uv.max.x, spr_uv.min.y);
    } else if index == 2 {
        uv = vec2(spr_uv.min.x, spr_uv.max.y);
    } else if index == 3 {
        uv = spr_uv.max;
    }
    let uv_dimensions = spr_uv.max - spr_uv.min;
#else
    var uv: vec2<f32>;
    if index == 0 {
        uv = vec2(1.);
    }
#endif


    var dimensions = vec2<f32>(textureDimensions(spr_texture));
    let position = vec4<f32>(
        in.position.xy * dimensions * uv_dimensions,
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
        vertex_output.uv = vec2<f32>(1. - uv.x, uv.y);
    } else {
        vertex_output.uv = uv;
    }

    return vertex_output;
}
